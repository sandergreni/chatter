
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

	pub fn strip_crlf(input: &mut String)
	{
		if input.ends_with('\n') || input.ends_with('\r')
		{
			input.pop();
			if input.ends_with('\n') || input.ends_with('\r')
			{
				input.pop();
			}
		}
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

// ************************************************************************************************
// ********************************************* TESTS ********************************************
// ************************************************************************************************

#[cfg(test)]
mod tests
{
    use crate::util::strip_crlf;

	#[test]
	fn test_no_crlf()
	{
		let str = "test".to_owned();
		let mut copy = str.clone();
		strip_crlf(&mut copy);

		assert_eq!(str, copy);
	}
	
	#[test]
	fn test_with_cr()
	{
		let str = "test".to_owned();
		let mut copy = "test\r".to_owned();
		strip_crlf(&mut copy);

		assert_eq!(str, copy);
	}

	#[test]
	fn test_with_lf()
	{
		let str = "test".to_owned();
		let mut copy = "test\n".to_owned();
		strip_crlf(&mut copy);

		assert_eq!(str, copy);
	}
	
	#[test]
	fn test_with_crlf()
	{
		let str = "test".to_owned();
		let mut copy = "test\r\n".to_owned();
		strip_crlf(&mut copy);

		assert_eq!(str, copy);
	}
}