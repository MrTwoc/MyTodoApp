use crate::components::card::Card;
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
    children: Children,
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

    view! {
        <Card
            title=team.team_name.clone()
            subtitle=format!("{} members", member_count)
            interactive=interactive
            elevated=matches!(variant, TeamCardVariant::Elevated)
            on_click=on_click
        >
            <div class=("team-card", true) class=(variant_class, true)>
                <p class="team-card-desc">{description}</p>
                <p class="team-card-meta">
                    { "Leader: " }
                    <span class="team-card-meta-value">{team.team_leader_id.to_string()}</span>
                </p>
                {if team.team_settings.team_member_limit > 0 {
                    view! {
                        <p class="team-card-meta">
                            { "Limit: " }
                            <span class="team-card-meta-value">{team.team_settings.team_member_limit.to_string()}</span>
                        </p>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {children()}
            </div>
        </Card>
    }
}
