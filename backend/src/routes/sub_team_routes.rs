use salvo::prelude::*;

use crate::handlers::sub_team_handler;
use crate::middleware;

pub fn sub_team_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/teams")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{team_id}")
                .push(Router::with_path("subteams").post(sub_team_handler::create_sub_team).get(sub_team_handler::list_sub_teams)),
        )
        .push(
            Router::with_path("{team_id}")
                .push(Router::with_path("subteams/{sub_team_id}"))
        )
}

pub fn sub_team_single_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/subteams")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{sub_team_id}")
                .get(sub_team_handler::get_sub_team)
                .put(sub_team_handler::update_sub_team)
                .delete(sub_team_handler::delete_sub_team)
                .push(Router::with_path("members").get(sub_team_handler::list_sub_team_members))
                .push(
                    Router::with_path("members/{user_id}")
                        .delete(sub_team_handler::remove_sub_team_member),
                )
                .push(
                    Router::with_path("members").post(sub_team_handler::add_sub_team_member),
                )
                .push(
                    Router::with_path("members/{user_id}/role").put(
                        sub_team_handler::update_sub_team_member_level,
                    ),
                ),
        )
}
