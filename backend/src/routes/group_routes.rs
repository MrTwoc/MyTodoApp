use salvo::prelude::*;

use crate::handlers::group_handler;
use crate::middleware;

pub fn group_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/teams")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{team_id}").push(
                Router::with_path("groups")
                    .post(group_handler::create_group)
                    .get(group_handler::list_groups),
            ),
        )
        .push(Router::with_path("{team_id}").push(Router::with_path("groups/{group_id}")))
}

pub fn group_single_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/groups").hoop(auth_middleware).push(
        Router::with_path("{group_id}")
            .get(group_handler::get_group)
            .put(group_handler::update_group)
            .delete(group_handler::delete_group)
            .push(Router::with_path("members").get(group_handler::list_group_members))
            .push(Router::with_path("members/{user_id}").delete(group_handler::remove_group_member))
            .push(Router::with_path("members").post(group_handler::add_group_member))
            .push(
                Router::with_path("members/{user_id}/role")
                    .put(group_handler::update_group_member_level),
            ),
    )
}
