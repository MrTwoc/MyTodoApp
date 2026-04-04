use crate::store::team_store::Team;
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum TeamCardVariant {
    Default,
    Compact,
    Elevated,
}

#[component]
pub fn TeamCard(
    team: Team,
    #[prop(default = TeamCardVariant::Default)] variant: TeamCardVariant,
    #[prop(default = false)] interactive: bool,
    #[prop(default = Callback::new(|_| ()))] on_click: Callback<(ev::MouseEvent,)>,
) -> impl IntoView {
    let variant_class = match variant {
        TeamCardVariant::Default => "team-card-default",
        TeamCardVariant::Compact => "team-card-compact",
        TeamCardVariant::Elevated => "team-card-elevated",
    };

    let description = team
        .team_settings
        .team_description
        .unwrap_or_else(|| "No description".to_string());
    let member_count = team.team_members.len();
    let member_limit = team.team_settings.team_member_limit;

    view! {
        <div
            class=("team-card-wrapper", true)
            class=(variant_class, true)
            class=("team-card-interactive", interactive)
            on:click=move |ev| on_click.run((ev,))
        >
            <div class="team-card-icon">
                {team.team_name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()}
            </div>
            <div class="team-card-content">
                <h3 class="team-card-title">{team.team_name.clone()}</h3>
                <p class="team-card-desc">{description}</p>
                <div class="team-card-stats">
                    <div class="team-stat">
                        <span class="team-stat-value">{member_count}</span>
                        <span class="team-stat-label">"members"</span>
                    </div>
                    {if member_limit > 0 {
                        view! {
                            <div class="team-stat">
                                <span class="team-stat-value">{member_limit}</span>
                                <span class="team-stat-label">"limit"</span>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </div>
            </div>
            <div class="team-card-arrow">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M9 18l6-6-6-6"/>
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn TeamCardSkeleton() -> impl IntoView {
    view! {
        <div class="team-card-wrapper team-card-skeleton">
            <div class="skeleton" style="gap: 16px">
                <div class="skeleton-item skeleton-rect" style="width: 48px; height: 48px; border-radius: 50%"></div>
                <div style="flex: 1">
                    <div class="skeleton-item skeleton-rect" style="width: 60%; height: 20px"></div>
                    <div class="skeleton-item skeleton-rect" style="width: 100%; height: 16px; margin-top: 8px"></div>
                    <div class="skeleton-item skeleton-rect" style="width: 40%; height: 14px; margin-top: 8px"></div>
                </div>
            </div>
        </div>
    }
}
