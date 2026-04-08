use crate::db::db_task::DbTask;
use crate::db::db_user::DbUser;
use crate::models::task::{Task, TaskStatus};
use crate::models::team::Team;
use crate::services::team_service::TeamService;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TaskStatusStats {
    pub total: u32,
    pub active: u32,
    pub completed: u32,
    pub paused: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TeamTaskStat {
    pub team_id: u64,
    pub team_name: String,
    pub total: u32,
    pub active: u32,
    pub completed: u32,
    pub paused: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DashboardOverview {
    pub username: Option<String>,
    pub personal_tasks: TaskStatusStats,
    pub team_tasks: TaskStatusStats,
    pub recent_personal_tasks: Vec<Task>,
    pub recent_team_tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DashboardTaskStats {
    pub personal_tasks: TaskStatusStats,
    pub team_tasks: TaskStatusStats,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DashboardTeamStats {
    pub teams: Vec<TeamTaskStat>,
}

pub struct DashboardService;

impl DashboardService {
    pub async fn overview(pool: &PgPool, user_id: u64) -> Result<DashboardOverview> {
        let team_ids = DbUser::get_user_teams(pool, user_id)
            .await?
            .into_iter()
            .map(|id| id as u64)
            .collect::<Vec<u64>>();

        let personal_tasks = list_tasks_with_stats(
            DbTask::list_tasks(
                pool,
                Some(user_id),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
            )
            .await?,
        );

        let mut all_team_tasks = Vec::<Task>::new();
        let mut team_stats = TaskStatusStats::default();

        for team_id in &team_ids {
            let tasks = DbTask::list_tasks(
                pool,
                None,
                Some(*team_id),
                None,
                None,
                None,
                None,
                None,
                None,
                false,
            )
            .await?;
            team_stats = accumulate_stats(&team_stats, &tasks);
            all_team_tasks.extend(tasks);
        }

        let username = match TeamService::get_team(pool, team_ids.first().copied().unwrap_or_default())
            .await?
        {
            Some(team) => Some(team.team_name),
            None => None,
        };

        let mut recent_personal = DbTask::list_tasks(
            pool,
            Some(user_id),
            None,
            None,
            None,
            None,
            None,
            Some(10),
            None,
            false,
        )
        .await?;
        // keep deterministic order and trim explicit fields for frontend list size
        recent_personal.truncate(10);

        let mut recent_team = all_team_tasks;
        recent_team.sort_by_key(|task| std::cmp::Reverse(task.task_create_time));
        if recent_team.len() > 10 {
            recent_team.truncate(10);
        }

        Ok(DashboardOverview {
            username: username,
            personal_tasks,
            team_tasks: team_stats,
            recent_personal_tasks: recent_personal,
            recent_team_tasks: recent_team,
        })
    }

    pub async fn tasks(pool: &PgPool, user_id: u64) -> Result<DashboardTaskStats> {
        let personal_tasks = list_tasks_with_stats(
            DbTask::list_tasks(pool, Some(user_id), None, None, None, None, None, None, None, false).await?,
        );

        let mut team_stats = TaskStatusStats::default();
        let team_ids = DbUser::get_user_teams(pool, user_id)
            .await?
            .into_iter()
            .map(|id| id as u64)
            .collect::<Vec<u64>>();
        for team_id in team_ids {
            let tasks = DbTask::list_tasks(
                pool,
                None,
                Some(team_id),
                None,
                None,
                None,
                None,
                None,
                None,
                false,
            )
            .await?;
            team_stats = accumulate_stats(&team_stats, &tasks);
        }

        Ok(DashboardTaskStats {
            personal_tasks,
            team_tasks: team_stats,
        })
    }

    pub async fn teams(pool: &PgPool, user_id: u64) -> Result<DashboardTeamStats> {
        let teams = TeamService::list_teams(pool, None, Some(user_id)).await?;

        let mut output = DashboardTeamStats::default();
        output.teams.reserve(teams.len());

        for team in teams {
            let tasks = DbTask::list_tasks(
                pool,
                None,
                Some(team.team_id),
                None,
                None,
                None,
                None,
                None,
                None,
                false,
            )
            .await?;
            output.teams.push(stat_for_team(team, &tasks));
        }

        output.teams
            .sort_by(|a, b| a.team_name.cmp(&b.team_name));

        Ok(output)
    }
}

fn accumulate_stats(current: &TaskStatusStats, tasks: &[Task]) -> TaskStatusStats {
    tasks.iter().fold(current.clone(), |mut stats, task| {
        stats.total = stats.total.saturating_add(1);
        match task.task_status {
            TaskStatus::Active => stats.active = stats.active.saturating_add(1),
            TaskStatus::Completed => stats.completed = stats.completed.saturating_add(1),
            TaskStatus::Paused => stats.paused = stats.paused.saturating_add(1),
        }
        stats
    })
}

fn list_tasks_with_stats(tasks: Vec<Task>) -> TaskStatusStats {
    accumulate_stats(&TaskStatusStats::default(), &tasks)
}

fn stat_for_team(team: Team, tasks: &[Task]) -> TeamTaskStat {
    let stats = list_tasks_with_stats(tasks.to_vec());
    TeamTaskStat {
        team_id: team.team_id,
        team_name: team.team_name,
        total: stats.total,
        active: stats.active,
        completed: stats.completed,
        paused: stats.paused,
    }
}
