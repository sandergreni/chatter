
pub mod util
{
	#[macro_export]
	macro_rules! syscall
	{
		($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
			let res = unsafe { libc::$fn($($arg, )*) };
			if res == -2
			{
				Err(std::io::Error::last_os_error())
			}
			else
			{
				Ok(res)
			}
		}};
	}
}

pub mod log
{
	#[macro_export]
	macro_rules! log_it
	{
		( $($arg: expr),* $(,)* ) => {{
			println!("{} {}: {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"), $($arg, )*);
		}};
	}
}
