
use common::syscall;
use common::log_it;
use std::os::fd::{ AsRawFd, RawFd };
use std::net::TcpListener;
use std::mem;

fn read_event_create(key: RawFd) -> libc::epoll_event
{
	libc::epoll_event
	{
		events: (libc::EPOLLET | libc::EPOLLIN) as u32,
		u64: key.try_into().unwrap(),
	}
}

fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event)
{
	match syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))
	{
		Ok(_) => (),
		Err(error) => panic!("Unable to add interest: {:#?}", error),
	}
}

fn modify_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event)
{
    match syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event))
	{
		Ok(_) => (),
		Err(error) => panic!("Unable to modify interest: {:#?}", error),
	}
}

fn raw_accept(listen_fd: RawFd) -> Result<RawFd, std::io::Error>
{
	let mut storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
	let mut len = mem::size_of_val(&storage) as libc::socklen_t;
	match syscall!(accept(listen_fd, (&raw mut storage) as *mut _, &mut len))
	{
		Ok(fd) =>
		{
			//let addr = std::sys_common::net::sockaddr_to_addr(&storage, len as usize).unwrap();
			//log_it!("new client", addr);
			log_it!("new_client", fd);
			Ok(fd)
		},
		Err(error) =>
		{
			Err(error)
		}
	}
}

fn raw_recv(fd: RawFd, buffer: &mut[u8], size: usize) -> Result<usize, std::io::Error>
{
	match syscall!(recv(fd, buffer.as_mut().as_mut_ptr() as *mut libc::c_void, size, 0))
	{
		Ok(bytes_read) => Ok(bytes_read as usize),
		Err(error) => Err(error)
	}
}

fn raw_write(fd: RawFd, buffer: &[u8], size: usize) -> Result<usize, std::io::Error>
{
	match syscall!(send(fd, buffer.as_ptr() as *const libc::c_void, size, 0))
	{
		Ok(bytes_written) => Ok(bytes_written as usize),
		Err(error) => Err(error)
	}
}

#[derive(PartialEq)]
enum SocketMode
{
	Recv,
	Send,
	Ignore,
}

fn get_socket_mode(event: libc::epoll_event) -> SocketMode
{
	let evs= event.events;
	if libc::EPOLLIN == evs as i32 & libc::EPOLLIN 
	{
		SocketMode::Recv
	}
	else if libc::EPOLLOUT == evs as i32 & libc::EPOLLOUT
	{
		SocketMode::Send
	}
	else
	{
		log_it!("Unknown socket mode", evs);
		SocketMode::Ignore
	}
}

pub struct Epoller<Func>
where Func: FnMut(RawFd, Vec<u8>) -> (RawFd, String)
{
	epoll_fd: RawFd,
	tcp_listener: TcpListener,
	callback: Func,
}

impl<Func> Epoller<Func>
where Func: FnMut(RawFd, Vec<u8>) -> (RawFd, String)
{
	pub fn new(port: u16, cb: Func) -> Self
	{
		let fd: RawFd = match syscall!(epoll_create1(libc::EPOLL_CLOEXEC))
		{
			Ok(res) => res,
			Err(error) => panic!("Unable to create Epoller: {:#?}", error),
		};

		let addr = "0.0.0.0:".to_owned() + &port.to_string();

		let listener: TcpListener = match TcpListener::bind(addr)
		{
			Ok(res) =>
			{
				res.set_nonblocking(true).unwrap();
				res
			}
			Err(error) =>
			{
				panic!("Unable to bind socket: {:#?}", error);
			}
		};

		add_interest(fd, listener.as_raw_fd(), read_event_create(listener.as_raw_fd()));

		Self{ epoll_fd: fd, tcp_listener: listener, callback: cb }
	}
	
	fn add(&self, fd: RawFd)
	{
		add_interest(self.epoll_fd, fd, read_event_create(fd));
	}

	fn get_events(&self) -> Vec<libc::epoll_event>
	{
		let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
		let num_events = match syscall!(
			epoll_wait(
				self.epoll_fd,
				events.as_mut_ptr(),
				events.capacity().try_into().unwrap(),
				-1 as libc::c_int))
		{
			Ok(v) => v,
			Err(error) => panic!("Error during epoll_wait: {:#?}", error),
		};

		unsafe { events.set_len(num_events as usize) };

		events
	}

	fn accept(&mut self)
	{
		match raw_accept(self.tcp_listener.as_raw_fd())
		{
			Ok(fd) =>
			{
				let mut nonblocking = true as libc::c_int;
				syscall!(ioctl(fd, libc::FIONBIO, &mut nonblocking)).unwrap();
				self.add(fd);
			},
			Err(error) =>
			{
				panic!("Error during accept: {:#?}", error);
			}
		};

		modify_interest(self.epoll_fd, self.tcp_listener.as_raw_fd(), read_event_create(self.tcp_listener.as_raw_fd()));
	}

	fn handle_read(&mut self, conn_id: RawFd) -> Option<Vec<u8>>
	{
		let mut raw_buffer: [u8; 4096] = [0u8; 4096];
		match raw_recv(conn_id, &mut raw_buffer, 4096)
		{
			Ok(bytes_read) =>
			{
				return Some(raw_buffer[..bytes_read].to_vec());
			}
			Err(error) =>
			{
				log_it!("Unable to read from socket", error);
			}
		}
		
		None
	}

	fn handle_write(&mut self, conn_id: RawFd, msg: String)
	{
		if let Err(error) = raw_write(conn_id, msg.as_bytes(), msg.len())
		{
			log_it!("Unable to write to socket", error);
		}
	}

	pub fn start(&mut self)
	{
		loop
		{
			for event in self.get_events()
			{
				match event.u64.try_into()
				{
					Ok(conn_fd) =>
					{
						if self.tcp_listener.as_raw_fd() == conn_fd
						{
							self.accept();
						}
						else
						{
							match get_socket_mode(event)
							{
								SocketMode::Recv =>
								{
									if let Some(buffer) = self.handle_read(conn_fd)
									{
										let (write_fd, msg) = (self.callback)(conn_fd, buffer);
										self.handle_write(write_fd, msg);
									}
								},
								SocketMode::Send =>
								{
								},
								_ =>
								{
									panic!("Illegal socket mode");
								}
							}
						}
					}
					Err(error) => log_it!("Unable to cast from u64 to RawFd", error)
				}
			}
		}
	}
}

// ************************************************************************************************
// ********************************************* TESTS ********************************************
// ************************************************************************************************

#[cfg(test)]
mod tests
{
    use crate::{get_socket_mode, SocketMode};

	#[test]
	fn test_socket_mode_recv()
	{
		let event = libc::epoll_event
		{
			events: (libc::EPOLLET | libc::EPOLLIN) as u32,
			u64: 42,
		};

		assert!(SocketMode::Recv == get_socket_mode(event));
	}
	
	#[test]
	fn test_socket_mode_send()
	{
		let event = libc::epoll_event
		{
			events: (libc::EPOLLET | libc::EPOLLOUT) as u32,
			u64: 42,
		};

		assert!(SocketMode::Send == get_socket_mode(event));
	}

	#[test]
	fn test_socket_mode_ignore()
	{
		let event = libc::epoll_event
		{
			events: (libc::EPOLLET) as u32,
			u64: 42,
		};

		assert!(SocketMode::Ignore == get_socket_mode(event));
	}
}
