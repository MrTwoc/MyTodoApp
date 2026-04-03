use salvo::prelude::*;

use crate::handlers::dashboard_handler;
use crate::middleware;

pub fn dashboard_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/dashboard")
        .hoop(auth_middleware)
        .get(dashboard_handler::get_overview)
        .push(Router::with_path("tasks").get(dashboard_handler::get_task_overview))
        .push(Router::with_path("teams").get(dashboard_handler::get_team_overview))
}
