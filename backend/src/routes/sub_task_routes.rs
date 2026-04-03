use salvo::prelude::*;

use crate::handlers::sub_task_handler;
use crate::middleware;

pub fn sub_task_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/tasks")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{task_id}")
                .push(
                    Router::with_path("subtasks")
                        .post(sub_task_handler::create_sub_task)
                        .get(sub_task_handler::list_sub_tasks),
                )
                .push(
                    Router::with_path("subtasks/{sub_task_id}")
                        .put(sub_task_handler::update_sub_task)
                        .delete(sub_task_handler::delete_sub_task),
                ),
        )
}
