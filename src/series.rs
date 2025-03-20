use regex::Regex;

use std::sync::LazyLock;

static EPISODE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"S(?<season>\d{1,2})E(?<episode>\d{1,2})").unwrap());

pub fn find_episode(path: &str) -> SeriesEpisode {
    let caps = EPISODE_REGEX.captures(path).unwrap();
    SeriesEpisode {
        path: path.to_string(),
        season: caps["season"].parse().unwrap(),
        episode: caps["episode"].parse().unwrap(),
    }
}

static SERIES_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s*(?<name>(?:\s+|\w+)+)(?:\s*\(\s*(?<year>\s*\d{4})\s*\))?\s*").unwrap()
});

pub fn find_series(path: &str) -> CollectionSeries {
    let caps = SERIES_REGEX.captures(path).unwrap();
    CollectionSeries {
        path: path.to_string(),
        name: caps["name"].trim().to_string(),
        year: caps.name("year").map(|y| y.as_str().parse().unwrap()),
    }
}

static SEASON_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*?(?<season>\d+).*").unwrap());

pub fn find_season(path: &str) -> SeriesSeason {
    let caps = SEASON_REGEX.captures(path).unwrap();
    SeriesSeason {
        path: path.to_string(),
        season: caps["season"].parse().unwrap(),
    }
}

/// How content is organized in the [Collection].
pub enum CollectionType {
    /// Only individual items in the collection (eg. movies)
    Unique,
    /// Items are organized by seasons/episodes
    Series,
    /// Free organization where children roam free
    Folders,
}

/// A collection organized by seasons
pub struct CollectionSeries {
    #[allow(dead_code)]
    path: String,
    /// Name of the series
    #[allow(dead_code)]
    name: String,
    /// Year of the series
    #[allow(dead_code)]
    year: Option<u16>,
}

pub struct SeriesSeason {
    pub path: String,
    pub season: u8,
}

pub struct SeriesEpisode {
    pub path: String,
    pub season: u8,
    pub episode: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_name() {
        let s = "My awesome series";
        let series = find_series(s);

        assert_eq!(series.name, s);
        assert_eq!(series.year, None);
    }

    #[test]
    fn test_series_name_year() {
        let s = "My awesome poney ( 2020 ) ";
        let series = find_series(s);

        assert_eq!(series.name, "My awesome poney");
        assert_eq!(series.year, Some(2020));
    }

    #[test]
    fn test_season_name() {
        let s = "Season 1";
        let season = find_season(s);

        assert_eq!(season.season, 1);
    }

    #[test]
    fn test_season_with_zero() {
        let s = "Season 01";
        let season = find_season(s);

        assert_eq!(season.season, 1);
    }

    #[test]
    fn test_season_with_year() {
        let s = "Season 1 (2025)";
        let season = find_season(s);

        assert_eq!(season.season, 1);
    }
}
