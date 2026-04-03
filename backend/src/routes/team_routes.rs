use salvo::prelude::*;

use crate::handlers::team_handler;
use crate::middleware;

pub fn team_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/teams")
        .hoop(auth_middleware)
        .post(team_handler::create_team)
        .get(team_handler::list_teams)
        .push(
            Router::with_path("{team_id}")
                .get(team_handler::get_team)
                .put(team_handler::update_team)
                .delete(team_handler::delete_team)
                .push(
                    Router::with_path("members")
                        .post(team_handler::add_member)
                        .get(team_handler::get_members)
                        .push(
                            Router::with_path("{user_id}")
                                .delete(team_handler::remove_member)
                                .push(
                                    Router::with_path("role").put(team_handler::update_member_role),
                                ),
                        ),
                )
                .push(Router::with_path("invites").post(team_handler::create_invite))
                .push(
                    Router::with_path("join-requests")
                        .post(team_handler::create_join_request)
                        .push(
                            Router::with_path("{request_id}")
                                .put(team_handler::update_join_request_status),
                        ),
                )
                .push(Router::with_path("logs").get(team_handler::get_team_logs)),
        )
}
