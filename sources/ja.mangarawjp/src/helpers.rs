use aidoku::alloc::string::String;

// titles carry a trailing "Raw Free" and similar suffixes
pub fn clean_title(title: String) -> String {
	let suffixes = [" Raw Free", " Raw free", " raw free"];
	for suffix in suffixes {
		if let Some(clean) = title.strip_suffix(suffix) {
			return clean.trim().into();
		}
	}
	title
}

// chapter text looks like 【第N話】
pub fn extract_ch_number(s: &str) -> Option<f32> {
	let dai = '第';
	let wa = '話';

	let start = s.find(dai)? + dai.len_utf8();
	let end = s[start..].find(wa)? + start;

	let num_str = &s[start..end];
	num_str.parse().ok()
}

// the reader declares its ids in an inline script, e.g.
// <script>window.MangaId =  133 ;window.CNumber =  10 </script>
pub fn read_window_number(data: &str, name: &str, fractional: bool) -> Option<String> {
	let after = &data[data.find(name)? + name.len()..];
	let after_eq = after[after.find('=')? + 1..].trim_start();
	let end = after_eq
		.find(|c: char| !c.is_ascii_digit() && !(fractional && c == '.'))
		.unwrap_or(after_eq.len());
	let number = after_eq[..end].trim();
	(!number.is_empty()).then(|| number.into())
}
