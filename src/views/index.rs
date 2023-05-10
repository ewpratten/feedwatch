use chrono::DateTime;
use rss::Item;
use worker::Response;

use crate::models::subscriptions::Subscription;

struct RssFeedItem {
    pub subscription: Subscription,
    pub item: Item,
}

pub async fn render_index_page(subscriptions: &Vec<Subscription>) -> worker::Result<Response> {
    // Use the RSS library to fetch the feeds
    let mut feed_items = Vec::new();
    for subscription in subscriptions {
        let channel = subscription.get_channel().await.unwrap();
        for item in channel.items() {
            feed_items.push(RssFeedItem {
                subscription: subscription.clone(),
                item: item.clone(),
            });
        }
    }

    // Sort the feed items by date
    feed_items.sort_by(|a, b| {
        let date_a = DateTime::parse_from_rfc2822(a.item.pub_date().unwrap_or("")).unwrap();
        let date_b = DateTime::parse_from_rfc2822(b.item.pub_date().unwrap_or("")).unwrap();
        date_b.cmp(&date_a)
    });

    // Build the output HTML
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
            let mut output = String::new();
            for feed_item in feed_items {
                output.push_str(&format!(
                    r#"
                    <div class="feed-item">
                        <p style="border-left:0.25em solid lightgray;padding-left:0.5em">
                            <strong><a href="{}">{}</a></strong> - {}<br>
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
                ));
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
        headers
            .set("Cache-Control", "public, max-age=300, s-maxage=600")
            .unwrap();
        headers
    }))
}
