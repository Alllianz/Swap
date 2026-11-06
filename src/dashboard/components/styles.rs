pub fn get_dashboard_css() -> &'static str {
    r#"
:root {
  color-scheme: dark;
  --bg: #0b0e11; 
  --pod: #15191e; 
  --pod-grad: linear-gradient(180deg, #181d24 0%, #12151a 100%); 
  --inner: #0f1216; 
  --th: #1a2027;
  --line: rgba(255,255,255,.08); 
  --line-hi: rgba(255,255,255,.16);
  --fg: #EBE6E1; 
  --fg-hi: #FFFFFF; 
  --mid: #9FAAB5; 
  --dim: #6C7885;
  --pos: #00E676; 
  --neg: #FF2E4D; 
  --gold: #FFD700; 
  --btc: #F7931A;
  --sp500: #00B0FF;
  --sh-btc: #EF4444;
  --sh-gold: #F59E0B;
  --sh-sp500: #F43F5E;
  --cash: #10B981;
  --purple: #E040FB;
  --font: 'Montserrat', system-ui, -apple-system, sans-serif; 
  --mono: 'JetBrains Mono', Consolas, monospace;
}

* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: var(--bg); color: var(--fg); font: 500 14px/1.5 var(--font); min-height: 100vh; padding-bottom: 60px; }
.wrap { max-width: 1400px; margin: 0 auto; padding-inline: max(16px, env(safe-area-inset-left, 0px)); }

/* Header & Flag */
.strip { height: 4px; background: linear-gradient(90deg, var(--btc), var(--gold), var(--sp500), var(--cash)); box-shadow: 0 2px 12px rgba(0,176,255,0.3); }
header.hero { background: linear-gradient(180deg, rgba(21,25,30,0.85) 0%, rgba(11,14,17,1) 100%); border-bottom: 1px solid var(--line); padding-bottom: 18px; }
header.hero .wrap { padding-top: 24px; display: flex; flex-direction: column; gap: 12px; }
.hero-top { display: flex; justify-content: space-between; align-items: flex-start; gap: 20px; flex-wrap: wrap; }
.eyebrow { font-size: 11px; font-weight: 800; letter-spacing: .14em; text-transform: uppercase; color: var(--sp500); margin-bottom: 4px; }
h1 { font-size: clamp(22px, 3.2vw, 32px); font-weight: 900; color: var(--fg-hi); letter-spacing: -.02em; }
.lead { color: var(--mid); max-width: 90ch; font-size: 13.5px; margin-top: 6px; }
.nota { font-size: 12px; color: var(--dim); background: rgba(255,255,255,0.03); border-left: 3px solid var(--gold); padding: 8px 12px; border-radius: 4px; }

/* Badges */
.hero-badges { display: flex; gap: 10px; flex-wrap: wrap; }
.badge-item { background: var(--pod); border: 1px solid var(--line); border-radius: 8px; padding: 6px 12px; display: flex; flex-direction: column; gap: 2px; }
.badge-lbl { font-size: 10px; font-weight: 700; text-transform: uppercase; color: var(--dim); letter-spacing: .06em; }
.badge-val { font-size: 12.5px; font-weight: 700; color: var(--fg-hi); font-family: var(--mono); }
.badge-ok { color: var(--pos); }

/* Navigation */
nav.top { position: sticky; top: 0; z-index: 100; background: rgba(11,14,17,0.92); backdrop-filter: blur(12px); border-bottom: 1px solid var(--line); padding: 8px 0; }
nav.top .wrap { display: flex; justify-content: space-between; align-items: center; gap: 16px; overflow-x: auto; }
nav.top .links { display: flex; gap: 14px; white-space: nowrap; }
nav.top a { color: var(--mid); text-decoration: none; font-size: 12px; font-weight: 700; transition: color 0.2s; padding: 4px 6px; border-radius: 4px; }
nav.top a:hover { color: var(--fg-hi); background: rgba(255,255,255,0.04); }
.reloj { font-size: 11px; color: var(--dim); font-family: var(--mono); white-space: nowrap; }

/* Layout & Sections */
main { display: flex; flex-direction: column; gap: 32px; margin-top: 24px; }
section { display: flex; flex-direction: column; gap: 16px; scroll-margin-top: 60px; }
.section-head { display: flex; justify-content: space-between; align-items: flex-end; gap: 16px; border-bottom: 1px solid var(--line); padding-bottom: 8px; flex-wrap: wrap; }
.section-head h2 { font-size: 18px; font-weight: 800; color: var(--fg-hi); }
.section-head .sub { font-size: 12px; color: var(--mid); margin-top: 2px; }

/* Pod Container */
.pod { background: var(--pod); border: 1px solid var(--line); border-radius: 12px; padding: 18px; display: flex; flex-direction: column; gap: 16px; }

/* KPI Grids */
.kgrid { display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 8px; }
.kpi { background: var(--inner); border: 1px solid var(--line); border-radius: 8px; padding: 8px 10px; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.kpi .k { font-size: 10px; font-weight: 700; color: var(--dim); text-transform: uppercase; letter-spacing: .04em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.kpi .v { font-size: 15px; font-weight: 800; color: var(--fg-hi); font-variant-numeric: tabular-nums; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: var(--mono); }
.kpi .s { font-size: 10.5px; color: var(--mid); font-variant-numeric: tabular-nums; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

/* Pills & Badges */
.pill { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; font-weight: 700; padding: 3px 8px; border-radius: 6px; font-family: var(--mono); white-space: nowrap; }
.pill.gris { background: var(--inner); color: var(--fg); border: 1px solid var(--line); }
.pill.azul { background: rgba(0, 176, 255, 0.15); color: var(--sp500); border: 1px solid rgba(0,176,255,0.3); }
.pill.gold { background: rgba(255, 215, 0, 0.15); color: var(--gold); border: 1px solid rgba(255,215,0,0.3); }

/* SWAP Podium (Top 3) */
.podio { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 12px; margin-bottom: 4px; }
.podio .pod { background: var(--pod-grad); border: 1px solid var(--line); border-radius: 12px; padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; }
.podio .pod.p1 { border-color: rgba(255, 215, 0, 0.5); box-shadow: 0 0 16px rgba(255, 215, 0, 0.08); }
.podio .pod.p2 { border-color: rgba(200, 200, 200, 0.4); }
.podio .pod.p3 { border-color: rgba(205, 127, 50, 0.4); }
.cc-h { display: flex; justify-content: space-between; align-items: center; }
.rk { display: inline-block; padding: 2px 8px; border-radius: 6px; font-size: 11.5px; font-weight: 800; background: var(--inner); }
.rk.r1 { background: rgba(255, 215, 0, 0.2); color: #FFD700; border: 1px solid rgba(255, 215, 0, 0.3); }
.rk.r2 { background: rgba(200, 200, 200, 0.2); color: #E0E0E0; border: 1px solid rgba(200, 200, 200, 0.3); }
.rk.r3 { background: rgba(205, 127, 50, 0.2); color: #FFAB40; border: 1px solid rgba(205, 127, 50, 0.3); }

.pm { display: grid; grid-template-columns: 1fr 1fr; gap: 6px 12px; font-size: 12px; background: var(--inner); border: 1px solid var(--line); border-radius: 8px; padding: 8px 10px; }
.pm div { display: flex; flex-direction: column; gap: 2px; }
.pm .l { font-size: 9.5px; color: var(--dim); text-transform: uppercase; font-weight: 700; letter-spacing: .04em; }
.pm .v { font-size: 13.5px; font-weight: 800; font-family: var(--mono); color: var(--fg-hi); }

/* Control Bar */
.ctrl-bar { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 14px; background: rgba(0,0,0,0.25); padding: 10px 14px; border-radius: 8px; border: 1px solid var(--line); }
.grupo { display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 600; color: var(--mid); }
.seg { display: inline-flex; background: var(--inner); border: 1px solid var(--line); border-radius: 6px; padding: 2px; }
.seg button { background: transparent; border: none; color: var(--mid); padding: 5px 12px; font-size: 11.5px; font-weight: 700; border-radius: 4px; cursor: pointer; transition: all 0.2s; font-family: var(--font); }
.seg button:hover { color: var(--fg-hi); }
.seg button.active, .seg button.on { background: var(--sp500); color: #000; }

.select-box { background: var(--inner); border: 1px solid var(--line); color: var(--fg-hi); font-size: 12px; font-weight: 600; padding: 6px 12px; border-radius: 6px; outline: none; cursor: pointer; font-family: var(--font); }

/* Tabs */
.tabs { display: flex; gap: 4px; border-bottom: 1px solid var(--line); overflow-x: auto; }
.tab-btn { background: transparent; border: none; border-bottom: 2px solid transparent; color: var(--mid); padding: 8px 14px; font-size: 13px; font-weight: 700; cursor: pointer; transition: all 0.2s; white-space: nowrap; font-family: var(--font); }
.tab-btn:hover { color: var(--fg-hi); }
.tab-btn.active { color: var(--sp500); border-bottom-color: var(--sp500); }
.tab-panel { display: none; flex-direction: column; gap: 14px; }
.tab-panel.active { display: flex; }
.panel-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap; }
.panel-info h3 { font-size: 15px; margin-bottom: 4px; }
.panel-info p { font-size: 12px; color: var(--mid); max-width: 80ch; }

/* Charts */
.chart-container { position: relative; width: 100%; border: 1px solid var(--line); border-radius: 8px; background: rgba(0,0,0,0.2); padding: 12px; }
.chart-lg { height: 480px; }
.chart-md { height: 360px; }

/* Legends */
.legend-toggles { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.legend-item { display: inline-flex; align-items: center; gap: 6px; font-size: 11.5px; font-weight: 600; cursor: pointer; padding: 3px 8px; border-radius: 4px; border: 1px solid var(--line); background: var(--inner); transition: all 0.2s; }
.legend-item:hover { border-color: var(--mid); }
.legend-item.dimmed { opacity: 0.35; text-decoration: line-through; }
.legend-color { width: 10px; height: 10px; border-radius: 2px; }

/* Tables */
.tabla-wrap { overflow-x: auto; border: 1px solid var(--line); border-radius: 8px; background: var(--inner); }
.tabla { width: 100%; border-collapse: collapse; font-size: 12px; text-align: left; }
.tabla th { background: var(--th); color: var(--mid); font-weight: 700; text-transform: uppercase; font-size: 10.5px; letter-spacing: .06em; padding: 10px 12px; border-bottom: 1px solid var(--line); white-space: nowrap; }
.tabla td { padding: 9px 12px; border-bottom: 1px solid rgba(255,255,255,0.04); white-space: nowrap; }
.tabla tr:hover { background: rgba(255,255,255,0.03); }
.al-l { text-align: left; }
.al-r { text-align: right; }
.al-c { text-align: center; }

/* Dots and Asset Badges */
.badge-dot { display: inline-flex; align-items: center; gap: 4px; font-weight: 700; font-size: 11px; padding: 2px 6px; border-radius: 4px; font-family: var(--mono); }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; margin-right: 6px; }
.dot-btc { background: rgba(247, 147, 26, 0.15); color: #FFA726; border: 1px solid rgba(247,147,26,0.3); }
.dot-gold { background: rgba(255, 215, 0, 0.15); color: #FFD700; border: 1px solid rgba(255,215,0,0.3); }
.dot-sp500 { background: rgba(0, 176, 255, 0.15); color: #00B0FF; border: 1px solid rgba(0,176,255,0.3); }
.dot-sh-btc { background: rgba(239, 68, 68, 0.18); color: #F87171; border: 1px solid rgba(239,68,68,0.4); }
.dot-sh-gold { background: rgba(245, 158, 11, 0.18); color: #FBBF24; border: 1px solid rgba(245,158,11,0.4); }
.dot-sh-sp500 { background: rgba(244, 63, 94, 0.18); color: #FB7185; border: 1px solid rgba(244,63,94,0.4); }
.dot-cash { background: rgba(16, 185, 129, 0.15); color: #10B981; border: 1px solid rgba(16,185,129,0.3); }

/* Links */
.lnk { background: none; border: none; color: inherit; font: inherit; cursor: pointer; padding: 0; text-align: left; display: inline-flex; align-items: center; }
.lnk:hover { color: var(--sp500); }

/* Grids */
.grid-3col { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
@media (max-width: 1024px) { .grid-3col { grid-template-columns: 1fr; } }
@media (max-width: 768px) { .kgrid { grid-template-columns: 1fr; } }

.fitness-card { background: var(--inner); border: 1px solid var(--line); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 6px; }
.fitness-card h4 { font-size: 13px; font-weight: 800; color: var(--fg-hi); display: flex; justify-content: space-between; }
.fitness-card p { font-size: 11.5px; color: var(--mid); }
.fitness-weight { font-size: 11px; color: var(--sp500); font-weight: 700; }

.val-pos, .pos { color: var(--pos) !important; font-weight: 700; }
.val-neg, .neg { color: var(--neg) !important; font-weight: 700; }
.gold { color: var(--gold) !important; font-weight: 700; }
.tag-badge { font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 4px; background: rgba(0,176,255,0.15); color: var(--sp500); border: 1px solid rgba(0,176,255,0.3); }

/* Mano a Mano Comparador */
.h2h-ctrl-bar { display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap; background: rgba(0,0,0,0.35); padding: 12px 16px; border-radius: 8px; border: 1px solid var(--line); }
.h2h-select-grp { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 220px; }
.h2h-lbl { display: flex; align-items: center; font-size: 11.5px; font-weight: 700; white-space: nowrap; }
.h2h-tag { display: inline-block; padding: 3px 8px; border-radius: 4px; font-family: var(--mono); font-size: 11px; font-weight: 800; }
.tag-a { background: rgba(0, 230, 118, 0.2); color: #00E676; border: 1px solid rgba(0, 230, 118, 0.4); }
.tag-b { background: rgba(0, 176, 255, 0.2); color: #00B0FF; border: 1px solid rgba(0, 176, 255, 0.4); }
.tag-c { background: rgba(255, 171, 0, 0.2); color: #FFAB00; border: 1px solid rgba(255, 171, 0, 0.4); }
.h2h-select { width: 100%; min-width: 180px; font-size: 12.5px; }
.btn-swap { background: var(--inner); border: 1px solid var(--line); color: var(--fg-hi); padding: 7px 14px; border-radius: 6px; font-size: 12px; font-weight: 700; cursor: pointer; transition: all 0.2s; white-space: nowrap; font-family: var(--font); }
.btn-swap:hover { background: var(--line); color: var(--gold); border-color: var(--gold); }
.h2h-cards-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px; margin-top: 14px; }
@media (max-width: 768px) { .h2h-cards-grid { grid-template-columns: 1fr; } }
.h2h-card { background: var(--pod-grad); border: 1px solid var(--line); border-radius: 10px; padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; }
.h2h-card.card-a { border-color: rgba(0, 230, 118, 0.35); }
.h2h-card.card-b { border-color: rgba(0, 176, 255, 0.35); }
.h2h-card.card-c { border-color: rgba(255, 171, 0, 0.35); }
.h2h-card-head { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--line); padding-bottom: 8px; }
.h2h-card-title { font-size: 14px; font-weight: 800; color: var(--fg-hi); display: flex; align-items: center; gap: 6px; }
.h2h-kpis-mini { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px 10px; }
.adv-badge { display: inline-flex; align-items: center; gap: 4px; padding: 3px 8px; border-radius: 4px; font-family: var(--mono); font-size: 11px; font-weight: 800; white-space: nowrap; }
.adv-badge.adv-a { background: rgba(0, 230, 118, 0.18); color: #00E676; border: 1px solid rgba(0, 230, 118, 0.35); }
.adv-badge.adv-b { background: rgba(0, 176, 255, 0.18); color: #00B0FF; border: 1px solid rgba(0, 176, 255, 0.35); }
.adv-badge.adv-c { background: rgba(255, 171, 0, 0.18); color: #FFAB00; border: 1px solid rgba(255, 171, 0, 0.35); }
.adv-badge.adv-tie { background: rgba(255, 255, 255, 0.06); color: var(--mid); border: 1px solid var(--line); }
.h2h-cat-header { background: rgba(255,255,255,0.03); font-weight: 800; font-size: 11px; text-transform: uppercase; color: var(--gold); letter-spacing: .06em; padding: 8px 12px; }
"#
}
