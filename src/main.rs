use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs;

const START: &str = "<!-- RECENT_PRS_START -->";
const END: &str = "<!-- RECENT_PRS_END -->";
const GRAPHQL: &str = r#"
query($query: String!, $first: Int!, $after: String) {
  search(query: $query, type: ISSUE, first: $first, after: $after) {
    nodes {
      ... on PullRequest {
        title
        url
        number
        mergedAt
        repository {
          nameWithOwner
        }
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
"#;

#[derive(Debug)]
struct Config {
    readme: String,
    login: String,
    limit: usize,
    lookback_days: i64,
    max_pages: usize,
    check: bool,
}

#[derive(Debug, Deserialize)]
struct GraphqlResponse {
    data: Option<GraphqlData>,
    errors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GraphqlData {
    search: SearchResult,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    nodes: Vec<Option<PullRequest>>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct PullRequest {
    title: String,
    url: String,
    number: u64,
    #[serde(rename = "mergedAt")]
    merged_at: String,
    repository: Repository,
}

#[derive(Clone, Debug, Deserialize)]
struct Repository {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = parse_args()?;
    let token = env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .map_err(|_| "Set GITHUB_TOKEN or GH_TOKEN")?;

    let since = (Utc::now() - Duration::days(config.lookback_days))
        .date_naive()
        .format("%Y-%m-%d");
    let search_query = format!(
        "author:{} is:pr is:merged merged:>={} -user:{} sort:updated-desc",
        config.login, since, config.login
    );

    let mut prs = fetch_merged_prs(&token, &search_query, config.max_pages)?;
    prs.sort_by(|left, right| right.merged_at.cmp(&left.merged_at));
    prs.truncate(config.limit);

    let readme = fs::read_to_string(&config.readme)?;
    let updated = replace_block(&readme, &format_block(&prs, Utc::now()))?;

    if config.check {
        print!("{updated}");
    } else {
        fs::write(&config.readme, updated)?;
    }

    Ok(())
}

fn parse_args() -> Result<Config, Box<dyn Error>> {
    let mut config = Config {
        readme: "README.md".to_string(),
        login: env::var("GH_LOGIN").unwrap_or_else(|_| "bioinformatist".to_string()),
        limit: env_usize("PR_LIMIT", 6)?,
        lookback_days: env_i64("PR_LOOKBACK_DAYS", 365)?,
        max_pages: env_usize("PR_SEARCH_MAX_PAGES", 10)?,
        check: false,
    };

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--readme" => config.readme = next_value(&mut args, "--readme")?,
            "--login" => config.login = next_value(&mut args, "--login")?,
            "--limit" => config.limit = next_value(&mut args, "--limit")?.parse()?,
            "--lookback-days" => {
                config.lookback_days = next_value(&mut args, "--lookback-days")?.parse()?
            }
            "--max-pages" => config.max_pages = next_value(&mut args, "--max-pages")?.parse()?,
            "--check" => config.check = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {arg}").into()),
        }
    }

    Ok(config)
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("Missing value for {flag}").into())
}

fn env_usize(name: &str, default: usize) -> Result<usize, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(_) => Ok(default),
    }
}

fn env_i64(name: &str, default: i64) -> Result<i64, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(_) => Ok(default),
    }
}

fn print_help() {
    println!(
        "Usage: profile-readme [--readme README.md] [--login bioinformatist] \\
         [--limit 6] [--lookback-days 365] [--max-pages 10] [--check]"
    );
}

fn fetch_merged_prs(
    token: &str,
    search_query: &str,
    max_pages: usize,
) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let agent = ureq::AgentBuilder::new().build();
    let mut prs = Vec::new();
    let mut after: Option<String> = None;

    for _ in 0..max_pages {
        let response: GraphqlResponse = agent
            .post("https://api.github.com/graphql")
            .set("Authorization", &format!("Bearer {token}"))
            .set("Content-Type", "application/json")
            .set("User-Agent", "bioinformatist-profile-readme")
            .send_json(json!({
                "query": GRAPHQL,
                "variables": {
                    "query": search_query,
                    "first": 100,
                    "after": after,
                }
            }))?
            .into_json()?;

        if let Some(errors) = response.errors {
            return Err(format!("GitHub API errors: {errors}").into());
        }

        let search = response
            .data
            .ok_or("GitHub API response did not include data")?
            .search;

        prs.extend(search.nodes.into_iter().flatten());

        if !search.page_info.has_next_page {
            break;
        }

        after = search.page_info.end_cursor;
    }

    Ok(prs)
}

fn format_block(prs: &[PullRequest], generated_at: DateTime<Utc>) -> String {
    let body = if prs.is_empty() {
        "_No merged pull requests found._".to_string()
    } else {
        prs.iter()
            .map(|pr| {
                let title = pr.title.replace('\n', " ");
                let merged = pr.merged_at.get(..10).unwrap_or(&pr.merged_at);
                format!(
                    "- [{}#{}]({}) · {} · merged {}",
                    pr.repository.name_with_owner, pr.number, pr.url, title, merged
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{START}\n{body}\n\n_Last updated: {}_\n{END}",
        generated_at.format("%Y-%m-%d %H:%M UTC")
    )
}

fn replace_block(readme: &str, block: &str) -> Result<String, Box<dyn Error>> {
    let (before, rest) = readme
        .split_once(START)
        .ok_or_else(|| format!("README must contain {START}"))?;
    let (_, after) = rest
        .split_once(END)
        .ok_or_else(|| format!("README must contain {END}"))?;
    Ok(format!("{before}{block}{after}"))
}
