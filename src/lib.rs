const KIB: u64 = 1024;
const MIB: u64 = KIB.pow(2);
const GIB: u64 = KIB.pow(3);
const TIB: u64 = KIB.pow(4);

pub fn format_bytes(bytes: u64, width: Option<usize>, decimals: Option<usize>) -> String {
	let width = width.unwrap_or(6);
	let decimals = decimals.unwrap_or(2);
	let strlen = bytes.to_string().len(); 
	let format_string = match strlen {
		0..=4 => {
			format!("{:>1$} B",bytes,width)
		},
		5..=7 => {
			format!("{:>1$.2$} KiB",bytes as f64/KIB as f64,width,decimals)
		},
		8..=10 => {
			format!("{:>1$.2$} MiB",bytes as f64/MIB as f64,width,decimals)
		},
		11..=13 => {
			format!("{:>1$.2$} GiB",bytes as f64/GIB as f64,width,decimals)
		},
		_ => {
			format!("{:>1$.2$} TiB",bytes as f64/TIB as f64,width,decimals)
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
    fn format_mebibytes() {
        let result = format_bytes(10485760, None, None);
        assert_eq!(result.as_str(), " 10.00 MiB");
    }
    #[test]
    fn format_gibibytes() {
        let result = format_bytes(53687091200, None, None);
        assert_eq!(result.as_str(), " 50.00 GiB");
    }
}
