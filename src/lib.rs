mod models;
mod views;

use models::subscriptions::get_subscriptions;
use views::index::render_index_page;
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Router
    let router = Router::new();

    // Set up routes
    router
        // Index page
        .get_async("/", |_req, _ctx| async move {
            let subscriptions = get_subscriptions();
            render_index_page(&subscriptions, None).await
        })
        // Tag-specific pages
        .get_async("/tag/:tag", |_req, ctx| async move {
            let subscriptions = get_subscriptions();
            let tag = ctx.param("tag").unwrap();
            render_index_page(&subscriptions, Some(vec![tag.to_string()])).await
        })
        .run(req, env)
        .await
}
