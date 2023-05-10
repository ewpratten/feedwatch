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
        .get_async("/", |req, _ctx| async move {
            let subscriptions = get_subscriptions();
            render_index_page(&subscriptions).await
        })
        .run(req, env)
        .await
}
