pub mod client_script;
pub mod compare_charts;
pub mod detail_kpis;
pub mod detail_trades;
pub mod head_to_head;
pub mod header;
pub mod leaderboard;
pub mod styles;

pub use client_script::get_client_javascript;
pub use compare_charts::render_compare_charts_section;
pub use detail_kpis::render_detail_kpis_section;
pub use detail_trades::render_detail_trades_section;
pub use head_to_head::render_head_to_head_section;
pub use header::render_header;
pub use leaderboard::render_leaderboard_section;
pub use styles::get_dashboard_css;
