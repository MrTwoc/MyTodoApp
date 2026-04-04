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
    let team_id = team.team_id;

    let initials: String = team
        .team_name
        .split_whitespace()
        .take(2)
        .map(|w| {
            w.chars()
                .next()
                .map(|c| c.to_ascii_uppercase())
                .unwrap_or(' ')
        })
        .collect();

    view! {
        <div
            class=("team-card-wrapper", true)
            class=(variant_class, true)
            class=("team-card-interactive", interactive)
            on:click=move |ev| on_click.run((ev,))
        >
            <div class="team-card-icon">
                {initials}
            </div>
            <div class="team-card-content">
                <div class="team-card-header-row">
                    <h3 class="team-card-title">{team.team_name.clone()}</h3>
                    <span class="team-card-id">#{team_id}</span>
                </div>
                <p class="team-card-desc">{description}</p>
                <div class="team-card-stats">
                    <div class="team-stat">
                        <svg class="team-stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                            <circle cx="9" cy="7" r="4"/>
                            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                        </svg>
                        <span class="team-stat-value">{member_count}</span>
                        <span class="team-stat-label">"members"</span>
                    </div>
                    {if member_limit > 0 {
                        view! {
                            <div class="team-stat">
                                <svg class="team-stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/>
                                    <path d="M12 6v6l4 2"/>
                                </svg>
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
