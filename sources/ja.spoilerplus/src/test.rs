use super::*;
use aidoku::{DeepLinkHandler, Listing, ListingProvider, MangaPageResult};
use aidoku_test::aidoku_test;

const SERIES_KEY: &str = "/HUNTER X HUNTER-raw-free/";

fn source() -> WpComics<SpoilerPlus> {
	WpComics::new()
}

fn listing(id: &str, page: i32) -> MangaPageResult {
	source()
		.get_manga_list(
			Listing {
				id: id.into(),
				..Default::default()
			},
			page,
		)
		.expect("listing request should succeed")
}

fn series() -> Manga {
	source()
		.get_manga_update(
			Manga {
				key: SERIES_KEY.into(),
				..Default::default()
			},
			true,
			true,
		)
		.expect("details request should succeed")
}

fn leading_keys(result: &MangaPageResult) -> Vec<&String> {
	result
		.entries
		.iter()
		.take(5)
		.map(|manga| &manga.key)
		.collect()
}

#[aidoku_test]
fn test_listing_latest() {
	assert!(
		!listing("latest", 1).entries.is_empty(),
		"latest listing should return entries"
	);
}

#[aidoku_test]
fn test_listing_latest_page_2() {
	assert!(
		!listing("latest", 2).entries.is_empty(),
		"latest listing page 2 should return entries"
	);
}

#[aidoku_test]
fn test_listing_ranking() {
	assert!(
		!listing("ranking", 1).entries.is_empty(),
		"ranking listing should return entries"
	);
}

// the trending page has no pager, so the app must not be told to ask for more
#[aidoku_test]
fn test_listing_trending_is_a_single_page() {
	let result = listing("trending", 1);
	assert!(
		!result.entries.is_empty(),
		"trending listing should return entries"
	);
	assert!(!result.has_next_page, "trending should not paginate");
}

#[aidoku_test]
fn test_unknown_listing_errors() {
	assert!(
		source()
			.get_manga_list(
				Listing {
					id: "nope".into(),
					..Default::default()
				},
				1
			)
			.is_err(),
		"an unknown listing should not fall back to another path"
	);
}

#[aidoku_test]
fn test_browse_without_query() {
	let result = source()
		.get_search_manga_list(None, 1, Vec::new())
		.expect("browse request should succeed");
	assert!(!result.entries.is_empty(), "browse should return entries");
}

#[aidoku_test]
fn test_search() {
	let result = source()
		.get_search_manga_list(Some(String::from("ワンピース")), 1, Vec::new())
		.expect("search request should succeed");
	assert!(!result.entries.is_empty(), "search should return entries");
}

// the latest listing starts at the home page, which stacks a carousel and a
// ranking block around the paginated list
#[aidoku_test]
fn test_latest_page_1_holds_only_the_paginated_block() {
	let result = listing("latest", 1);

	let mut keys = result
		.entries
		.iter()
		.map(|manga| manga.key.as_str())
		.collect::<Vec<_>>();
	let total = keys.len();
	keys.sort_unstable();
	keys.dedup();
	assert_eq!(keys.len(), total, "a page should not repeat entries");

	// one block currently holds 24 entries
	assert!(
		total <= 40,
		"a page should hold a single block, got {total} entries"
	);
}

#[aidoku_test]
fn test_keys_stay_relative_and_covers_absolute() {
	let result = listing("latest", 1);

	for manga in &result.entries {
		assert!(
			manga.key.starts_with('/') && manga.key.ends_with("-raw-free/"),
			"key should be a site-relative series path, got {}",
			manga.key
		);
		let cover = manga.cover.as_ref().expect("entry should have a cover");
		assert!(
			cover.starts_with(BASE_URL),
			"cover should be an absolute url, got {cover}"
		);
	}
}

#[aidoku_test]
fn test_pagination_ends() {
	let result = listing("latest", 9999);
	assert!(
		result.entries.is_empty() && !result.has_next_page,
		"out of range page should end pagination"
	);
}

#[aidoku_test]
fn test_listings_return_different_orders() {
	let latest = listing("latest", 1);
	let trending = listing("trending", 1);
	let ranking = listing("ranking", 1);
	assert_ne!(
		leading_keys(&latest),
		leading_keys(&ranking),
		"latest and ranking should not resolve to the same order"
	);
	assert_ne!(
		leading_keys(&trending),
		leading_keys(&ranking),
		"trending and ranking should not resolve to the same order"
	);
}

#[aidoku_test]
fn test_manga_details() {
	let manga = series();
	assert_eq!(
		manga.title, "HUNTER X HUNTER",
		"title should drop the suffix"
	);
	let cover = manga.cover.expect("series should have a cover");
	assert!(
		cover.starts_with(BASE_URL),
		"cover should be an absolute url, got {cover}"
	);
	assert!(
		manga.description.is_some_and(|d| !d.is_empty()),
		"series should have a description"
	);
	assert!(
		manga.tags.is_some_and(|t| !t.is_empty()),
		"series should have tags"
	);
}

#[aidoku_test]
fn test_status_is_not_read_from_the_alternative_titles_row() {
	assert_eq!(
		series().status,
		MangaStatus::Ongoing,
		"a running series should not be read off the alternative titles row"
	);
}

#[aidoku_test]
fn test_chapter_list() {
	let chapters = series().chapters.expect("series should have chapters");
	assert!(chapters.len() >= 400, "got {} chapters", chapters.len());

	let first = chapters.first().expect("chapter list should not be empty");
	assert!(
		first.key.starts_with(SERIES_KEY),
		"chapter key should sit under the series path, got {}",
		first.key
	);
	assert!(
		first.chapter_number.is_some(),
		"chapter number should be parsed out of the title"
	);
}

// on the device `abs:href` collapses the raw utf-8 chapter segment to the series
// page, which carries no window.MangaId. the runner resolves abs urls with the
// `url` crate, which percent-encodes instead, so only the transformer itself can
// be checked here
#[aidoku_test]
fn test_chapter_urls_are_encoded() {
	assert_eq!(
		(SpoilerPlus.params().chapter_url_transformer)("/HUNTER X HUNTER-raw-free/第417話/".into()),
		"https://spoilerplus.tv/HUNTER%20X%20HUNTER-raw-free/%E7%AC%AC417%E8%A9%B1/"
	);
}

// the app renders the number it is given, so a title repeating it shows twice
#[aidoku_test]
fn test_numbered_chapters_carry_no_title() {
	let chapters = series().chapters.expect("series should have chapters");

	for chapter in chapters.iter().take(20) {
		if chapter.chapter_number.is_none() {
			continue;
		}
		assert!(
			chapter.title.is_none(),
			"a numbered chapter should not repeat its number in the title, got {:?} / {:?}",
			chapter.chapter_number,
			chapter.title
		);
	}
}

#[aidoku_test]
fn test_page_list() {
	let chapters = series().chapters.expect("series should have chapters");
	let chapter = chapters
		.into_iter()
		.next()
		.expect("chapter list should not be empty");
	let pages = source()
		.get_page_list(Manga::default(), chapter)
		.expect("page list request should succeed");

	assert!(!pages.is_empty(), "chapter should have pages");
	for page in &pages {
		let PageContent::Url(url, context) = &page.content else {
			panic!("page should be a url");
		};
		assert!(
			url.starts_with(IMG_CDN),
			"page should be served from the image cdn, got {url}"
		);
		// a missing key would silently ship scrambled images
		assert!(
			context
				.as_ref()
				.and_then(|c| c.get("key"))
				.is_some_and(|key| !key.is_empty()),
			"page should carry the descrambling key"
		);
	}
}

#[aidoku_test]
fn test_deep_links() {
	let handle = |url: &str| {
		source()
			.handle_deep_link(String::from(url))
			.expect("deep link should be handled")
	};

	assert_eq!(
		handle("https://spoilerplus.tv/HUNTER%20X%20HUNTER-raw-free/"),
		Some(DeepLinkResult::Manga {
			key: SERIES_KEY.into()
		})
	);
	assert_eq!(
		handle("https://spoilerplus.tv/HUNTER%20X%20HUNTER-raw-free/第417話/"),
		Some(DeepLinkResult::Chapter {
			manga_key: SERIES_KEY.into(),
			key: "/HUNTER X HUNTER-raw-free/第417話/".into(),
		})
	);
	assert_eq!(handle("https://spoilerplus.tv/genre/Ecchi/"), None);
	assert_eq!(handle("https://spoilerplus.tv/ranking/"), None);
	assert_eq!(handle("https://example.com/HUNTER-raw-free/"), None);
}
