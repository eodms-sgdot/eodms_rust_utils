use num_traits::{PrimInt, ToPrimitive};

pub mod file;
pub mod signal;
pub mod dropbox;

const KIB: u64 = 1024;
const MIB: u64 = KIB.pow(2);
const GIB: u64 = KIB.pow(3);
const TIB: u64 = KIB.pow(4);
const PIB: u64 = KIB.pow(5);

pub fn format_bytes<T: ToPrimitive>(bytes: T, width: Option<usize>, decimals: Option<usize>) -> String {
	let width = width.unwrap_or(6);
	let decimals = decimals.unwrap_or(2);
	let float = match bytes.to_f64() {
		Some(f) => f,
		None => return "".to_string()
	};
	let strlen = (float as i64).to_string().len();
	let format_string = match strlen {
		0..=3 => {
			format!("{:>1$} B",float,width)
		},
		4..=6 => {
			format!("{:>1$.2$} KiB",float/KIB as f64,width,decimals)
		},
		7..=9 => {
			format!("{:>1$.2$} MiB",float/MIB as f64,width,decimals)
		},
		10..=12 => {
			format!("{:>1$.2$} GiB",float/GIB as f64,width,decimals)
		},
		13..=15 => {
			format!("{:>1$.2$} TiB",float/TIB as f64,width,decimals)
		},
		_ => {
			format!("{:>1$.2$} PiB",float/PIB as f64,width,decimals)
		},
	};
	format_string
}

pub fn format_rate<T: PrimInt>(bytes: T, duration: f64, width: Option<usize>, decimals: Option<usize>) -> String {
	let float = match bytes.to_f64() {
		Some(f) => f,
		None => return "".to_string()
	};
	let rate = match float / duration {
		x if x.is_infinite() => float,
		x => x,
	};
	let mut str = format_bytes(rate,width,decimals);
	str.push_str("/s");
	str
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn format_kibibytes() {
		let result = format_bytes(102400 as i32, None, None);
		assert_eq!(result.as_str(), "100.00 KiB");
	}
	#[test]
	fn format_kibibytes2() {
		let result = format_bytes(2048 as u16, None, None);
		assert_eq!(result.as_str(), "  2.00 KiB");
	}
	#[test]
	fn format_kibibytes_nodec() {
		let result = format_bytes(2048 as u16, Some(1), Some(0));
		assert_eq!(result.as_str(), "2 KiB");
	}
	#[test]
	fn format_mebibytes() {
		let result = format_bytes(10485760 as u32, None, None);
		assert_eq!(result.as_str(), " 10.00 MiB");
	}
	#[test]
	fn format_mebibytes2() {
		let result = format_bytes(999999999 as i32, None, None);
		assert_eq!(result.as_str(), "953.67 MiB");
	}
	#[test]
	fn format_gibibytes() {
		let result = format_bytes(53687091200 as i64, None, None);
		assert_eq!(result.as_str(), " 50.00 GiB");
	}
	#[test]
	fn format_gibibytes2() {
		let result = format_bytes(9999999999 as u64, None, None);
		assert_eq!(result.as_str(), "  9.31 GiB");
	}
	#[test]
	fn format_tebibytes() {
		let result = format_bytes(555555555555555 as u64, None, None);
		assert_eq!(result.as_str(), "505.27 TiB");
	}
	#[test]
	fn format_pebibytes() {
		let result = format_bytes(77777777777777777 as i64, None, None);
		assert_eq!(result.as_str(), " 69.08 PiB");
	}
	#[test]
	fn format_biggest_u64() {
		let result = format_bytes(18_446_744_073_709_551_615u64, None, None);
		assert_eq!(result.as_str(), "16384.00 PiB");
	}
	#[test]
	fn format_rate_test1() {
		let result = format_rate(1000000,10.0, None, None);
		assert_eq!(result.as_str(), " 97.66 KiB/s");
	}
	#[test]
	fn format_rate_test2() {
		let result = format_rate(74000200,15.34, None, None);
		assert_eq!(result.as_str(), "  4.60 MiB/s");
	}
	#[test]
	fn format_rate_zerosecs() {
		let result = format_rate(888888888,0.0, None, None);
		assert_eq!(result.as_str(), "847.71 MiB/s");
	}
}
