use std::time::Duration;
use reqwest::{StatusCode, Response};
use std::fmt;
use chart::Chart;
use std::cmp;

#[derive(Debug)]
pub struct Fact {
    status: StatusCode,
    duration: Duration,
    content_length: usize,
}

impl Fact {
    pub fn record(resp: Response, duration: Duration) -> Fact {
        Fact {
            duration,
            status: resp.status(),
            content_length: 0,
        }
    }
}

#[derive(Debug)]
pub struct Summary {
    average: Duration,
    median: Duration,
    max: Duration,
    min: Duration,
    count: u32,
    percentiles: Vec<Duration>,
    latency_histogram: Vec<u32>,
}

impl Summary {
    fn zero() -> Summary {
        Summary {
            average: Duration::new(0, 0),
            median: Duration::new(0, 0),
            max: Duration::new(0, 0),
            min: Duration::new(0, 0),
            count: 0,
            percentiles: vec![Duration::new(0, 0); 100],
            latency_histogram: vec![0; 0],
        }
    }
}

fn to_ms(d: Duration) -> f64 {
    (d.as_secs() as f64 * 1_000f64) + (d.subsec_nanos() as f64 / 1_000_000f64)
}

#[test]
fn exchange_duration_to_ms() {
    assert_eq!(to_ms(Duration::new(1, 500000)), 1000.5f64);
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Summary")?;
        writeln!(f, "  Average:   {} ms", to_ms(self.average))?;
        writeln!(f, "  Median:    {} ms", to_ms(self.median))?;
        writeln!(f, "  Longest:   {} ms", to_ms(self.max))?;
        writeln!(f, "  Shortest:  {} ms", to_ms(self.min))?;
        writeln!(f, "  Requests:  {}", self.count)?;
        writeln!(f, "")?;
        writeln!(f, "Latency Percentiles (2% of requests per bar):")?;
        let percentiles: Vec<f64> = self.percentiles.iter().map(|d| to_ms(*d)).collect();
        writeln!(f, "{}", Chart::new().make(&percentiles))?;
        writeln!(f, "")?;
        writeln!(f, "Latency Histogram (each bar is 2% of max latency)")?;
        writeln!(f, "{}", Chart::new().make(&self.latency_histogram))?;
        Ok(())
    }
}

impl Summary {
    pub fn from_facts(facts: &[Fact]) -> Summary {
        if facts.len() == 0 {
            return Summary::zero();
        }
        let count = facts.len() as u32;
        let sum: Duration = facts.iter().map(|f| f.duration).sum();
        let average = sum / count;
        let mut sorted: Vec<Duration> = facts.iter().map(|f| f.duration.clone()).collect();
        sorted.sort();

        let mid = sorted.len() / 2;
        let median = if facts.len() % 2 == 0 {
            // even
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            // odd
            sorted[mid]
        };
        let min = *sorted.first().expect("Returned early if empty");
        let max = *sorted.last().expect("Returned early if empty");

        let bin_size = to_ms(max) / 50.;
        let mut latency_histogram = vec![0; 50];

        for duration in &sorted {
            let index = (to_ms(*duration) / bin_size) as usize;
            latency_histogram[cmp::min(index, 49)] += 1;
        }

        let percentiles = (0..50)
            .map(|n| {
                let mut index = ((n as f64 / 50.0) * sorted.len() as f64) as usize;
                index = cmp::max(index, 0);
                index = cmp::min(index, sorted.len() - 1);
                sorted[index]
            })
            .collect();

        Summary {
            average,
            median,
            count,
            min,
            max,
            percentiles,
            latency_histogram,
        }
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn summarizes_to_zero_if_empty() {
        let summary = Summary::from_facts(&Vec::new());
        assert_eq!(summary.average, Duration::new(0, 0));
        assert_eq!(summary.median, Duration::new(0, 0));
        assert_eq!(summary.count, 0);
    }

    #[test]
    fn averages_the_durations() {
        let facts = [
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(1, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(2, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(3, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(4, 0),
                content_length: 0,
            },
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.average, Duration::new(2, 500000000));
    }

    #[test]
    fn counts_the_facts() {
        let facts = [
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(1, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(2, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(3, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(4, 0),
                content_length: 0,
            },
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.count, 4);
    }

    #[test]
    fn calculates_percentiles_from_an_even_number_of_facts() {
        let facts = [
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(1, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(2, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(3, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(100, 0),
                content_length: 0,
            },
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.median, Duration::new(2, 500000000));
        assert_eq!(summary.max, Duration::new(100, 0));
        assert_eq!(summary.min, Duration::new(1, 0));
    }

    #[test]
    fn calculates_percentiles_from_an_odd_number_of_facts() {
        let facts = [
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(1, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(2, 0),
                content_length: 0,
            },
            Fact {
                status: StatusCode::Ok,
                duration: Duration::new(100, 0),
                content_length: 0,
            },
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.median, Duration::new(2, 0));
        assert_eq!(summary.max, Duration::new(100, 0));
        assert_eq!(summary.min, Duration::new(1, 0));
    }

    #[test]
    fn counts_the_histogram_of_latencies() {
        let facts: Vec<Fact> = (0..500)
            .map(|n| {
                Fact {
                    status: StatusCode::Ok,
                    duration: Duration::new(n, 0),
                    content_length: 0,
                }
            })
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.latency_histogram.len(), 50);
        assert_eq!(summary.latency_histogram.first(), Some(&10));
        assert_eq!(summary.latency_histogram.last(), Some(&10));
        assert_eq!(summary.latency_histogram[25], 10);
    }

    #[test]
    fn calculates_all_the_percentiles_when_n_less_than_100() {
        let facts: Vec<Fact> = (0..50)
            .map(|n| {
                Fact {
                    status: StatusCode::Ok,
                    duration: Duration::new(n, 0),
                    content_length: 0,
                }
            })
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.percentiles.len(), 50);
        assert_eq!(summary.percentiles.first(), Some(&Duration::new(0, 0)));
        assert_eq!(summary.percentiles.last(), Some(&Duration::new(49, 0)));
        assert_eq!(summary.percentiles[25], Duration::new(25, 0));
    }

    #[test]
    fn calculates_all_the_percentiles_when_n_greater_than_100() {
        let facts: Vec<Fact> = (0..500)
            .map(|n| {
                Fact {
                    status: StatusCode::Ok,
                    duration: Duration::new(n, 0),
                    content_length: 0,
                }
            })
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.percentiles.len(), 50);
        assert_eq!(summary.percentiles.first(), Some(&Duration::new(0, 0)));
        assert_eq!(summary.percentiles.last(), Some(&Duration::new(490, 0)));
        assert_eq!(summary.percentiles[25], Duration::new(250, 0));
    }
}
