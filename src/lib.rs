const KIB: u64 = 1024;
const MIB: u64 = KIB.pow(2);
const GIB: u64 = KIB.pow(3);
const TIB: u64 = KIB.pow(4);
const PIB: u64 = KIB.pow(5);

pub fn format_bytes(bytes: u64, width: Option<usize>, decimals: Option<usize>) -> String {
	let width = width.unwrap_or(6);
	let decimals = decimals.unwrap_or(2);
	let strlen = bytes.to_string().len(); 
	let format_string = match strlen {
		0..=3 => {
			format!("{:>1$} B",bytes,width)
		},
		4..=6 => {
			format!("{:>1$.2$} KiB",bytes as f64/KIB as f64,width,decimals)
		},
		7..=9 => {
			format!("{:>1$.2$} MiB",bytes as f64/MIB as f64,width,decimals)
		},
		10..=12 => {
			format!("{:>1$.2$} GiB",bytes as f64/GIB as f64,width,decimals)
		},
		13..=15 => {
			format!("{:>1$.2$} TiB",bytes as f64/TIB as f64,width,decimals)
		},
		_ => {
			format!("{:>1$.2$} PiB",bytes as f64/PIB as f64,width,decimals)
		},
	};
	format_string
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn format_kibibytes() {
		let result = format_bytes(102400, None, None);
		assert_eq!(result.as_str(), "100.00 KiB");
	}
	#[test]
	fn format_kibibytes2() {
		let result = format_bytes(2048, None, None);
		assert_eq!(result.as_str(), "  2.00 KiB");
	}
	#[test]
	fn format_mebibytes() {
		let result = format_bytes(10485760, None, None);
		assert_eq!(result.as_str(), " 10.00 MiB");
	}
	#[test]
	fn format_mebibytes2() {
		let result = format_bytes(999999999, None, None);
		assert_eq!(result.as_str(), "953.67 MiB");
	}
	#[test]
	fn format_gibibytes() {
		let result = format_bytes(53687091200, None, None);
		assert_eq!(result.as_str(), " 50.00 GiB");
	}
	#[test]
	fn format_gibibytes2() {
		let result = format_bytes(9999999999, None, None);
		assert_eq!(result.as_str(), "  9.31 GiB");
	}
	#[test]
	fn format_tebibytes() {
		let result = format_bytes(555555555555555, None, None);
		assert_eq!(result.as_str(), "505.27 TiB");
	}
	#[test]
	fn format_pebibytes() {
		let result = format_bytes(77777777777777777, None, None);
		assert_eq!(result.as_str(), " 69.08 PiB");
	}
}
