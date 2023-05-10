use chrono::DateTime;
use itertools::Itertools;
use rss::Item;
use worker::{console_log, console_warn, Response};

use crate::models::subscriptions::Subscription;

struct RssFeedItem {
    pub subscription: Subscription,
    pub item: Item,
}

pub async fn render_index_page(
    subscriptions: &Vec<Subscription>,
    allowed_tags: Option<Vec<String>>,
) -> worker::Result<Response> {
    // Use the RSS library to fetch the feeds
    console_log!("Fetching feeds");
    let mut feed_items = Vec::new();
    for subscription in subscriptions {
        // Ignore dis-allowed tags
        if let Some(allowed_tags) = &allowed_tags {
            if !subscription
                .tags
                .iter()
                .any(|tag| allowed_tags.contains(tag))
            {
                continue;
            }
        }

        // Try to fetch the channel for the feed
        match subscription.get_channel().await{
            Ok(channel) => {
                for item in channel.items() {
                    feed_items.push(RssFeedItem {
                        subscription: subscription.clone(),
                        item: item.clone(),
                    });
                }
            },
            Err(e) => {
                console_warn!("Failed to fetch feed: {} ({})", subscription.url, e);
            }
        }
    }

    // Sort the feed items by date
    console_log!("Sorting feeds");
    feed_items.sort_by(|a, b| {
        let date_a = DateTime::parse_from_rfc2822(a.item.pub_date().unwrap_or("")).unwrap();
        let date_b = DateTime::parse_from_rfc2822(b.item.pub_date().unwrap_or("")).unwrap();
        date_b.cmp(&date_a)
    });

    // Build the output HTML
    console_log!("Building HTML");
    let output = textwrap::dedent(&format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta http-equiv="X-UA-Compatible" content="IE=edge">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Feed Watch</title>
            </head>
            <body>
                <div class="content" style="max-width:900px;width:100%;margin:auto">
                    <h1 style="margin-bottom:0">Aggregated RSS Feed Items</h1>
                    <p style="margin-top:0">This page collects interesting posts from around the internet</p>
                    {}
                    <hr>
                    <div class="feed-items">
                        {}
                    </div>
                </div>
                <div class="copyright" style="max-width:900px;width:100%;margin:auto;text-align:center">
                    <br><hr>
                    <p>
                        <strong>Copyright</strong> &copy; <a href="https://ewpratten.com">Evan Pratten</a>
                    </p>
                </div>
            </body>
        </html>
    "#,
        {
            match allowed_tags.is_some() {
                true => format!(
                    "<p><em><strong>Filtering by tags:</strong> {}</em></p>",
                    allowed_tags.unwrap().join(", ")
                ),
                false => textwrap::dedent(&format!(
                    r#"
                <p>
                    <em>
                        <strong>Filterable tags:</strong> {}
                    </em>
                </p>
                "#,
                    subscriptions
                        .iter()
                        .map(|sub| sub.tags.clone())
                        .flatten()
                        .unique()
                        .map(|tag| format!(r#"<a href="/tag/{}">{}</a>"#, tag, tag))
                        .collect::<Vec<String>>()
                        .join(", ")
                )),
            }
        },
        {
            let mut output = String::new();
            for feed_item in feed_items {
                output.push_str(&textwrap::dedent(&format!(
                    r#"
                    <div class="feed-item">
                        <p style="border-left:0.25em solid lightgray;padding-left:0.5em">
                            <strong><a href="{}" target="_blank">{}</a></strong> - {}<br>
                            <span style="color:gray">Published: {}</span>
                        </p>
                    </div>
                "#,
                    feed_item.item.link().unwrap_or(""),
                    feed_item.item.title().unwrap_or("NO TITLE"),
                    feed_item.subscription.name,
                    DateTime::parse_from_rfc2822(feed_item.item.pub_date().unwrap_or(""))
                        .map(|date| date.format("%Y-%m-%d").to_string())
                        .unwrap_or("UNKNOWN".to_string())
                )));
            }
            output
        }
    ));

    // Return the response
    Ok(Response::ok(output).unwrap().with_headers({
        let mut headers = worker::Headers::new();
        headers
            .set("Content-Type", "text/html; charset=utf-8")
            .unwrap();
        // Tell the browser to cache the page for 10 minutes
        headers
            .set("Cache-Control", "public, max-age=600")
            .unwrap();
        headers
    }))
}
