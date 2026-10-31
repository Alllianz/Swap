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
nav.top { position: sticky; top: 0; z-index: 50; background: rgba(11,14,17,.95); backdrop-filter: blur(8px); border-bottom: 1px solid var(--line); }
nav.top .wrap { display: flex; align-items: center; justify-content: space-between; padding-block: 8px; gap: 12px; }
nav.top .links { display: flex; gap: 6px; overflow-x: auto; }
nav.top a { color: var(--mid); text-decoration: none; font-size: 12.5px; font-weight: 700; padding: 6px 12px; border-radius: 6px; white-space: nowrap; transition: all 0.2s; }
nav.top a:hover { color: var(--fg-hi); background: rgba(255,255,255,.06); }
.reloj { font-size: 12px; font-weight: 700; color: var(--mid); white-space: nowrap; font-family: var(--mono); }

/* Main */
main.wrap { padding-block: 28px; display: flex; flex-direction: column; gap: 36px; }
section { scroll-margin-top: 60px; display: flex; flex-direction: column; gap: 14px; }
.section-head { display: flex; justify-content: space-between; align-items: baseline; flex-wrap: wrap; gap: 12px; }
h2 { font-size: 20px; font-weight: 800; color: var(--fg-hi); }
h3 { font-size: 15px; font-weight: 800; color: var(--fg-hi); }
.sub { color: var(--mid); font-size: 13px; max-width: 90ch; }

/* Pod */
.pod { background: var(--pod-grad); border: 1px solid var(--line); border-radius: 12px; padding: 18px; display: flex; flex-direction: column; gap: 14px; box-shadow: 0 4px 20px rgba(0,0,0,0.3); }
.kpod-h { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--line); padding-bottom: 8px; }

/* Control Bar */
.ctrl-bar { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 14px; background: rgba(0,0,0,0.25); padding: 10px 14px; border-radius: 8px; border: 1px solid var(--line); }
.grupo { display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 600; color: var(--mid); }
.seg { display: inline-flex; background: var(--inner); border: 1px solid var(--line); border-radius: 6px; padding: 2px; }
.seg button { background: transparent; border: none; color: var(--mid); padding: 4px 10px; font-size: 11.5px; font-weight: 700; border-radius: 4px; cursor: pointer; transition: all 0.2s; font-family: var(--font); }
.seg button:hover { color: var(--fg-hi); }
.seg button.active { background: var(--sp500); color: #000; }

.select-box { background: var(--inner); border: 1px solid var(--line); color: var(--fg-hi); font-size: 12px; font-weight: 600; padding: 5px 10px; border-radius: 6px; outline: none; cursor: pointer; font-family: var(--font); }

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
.dot-btc { background: rgba(247, 147, 26, 0.15); color: #FFA726; border: 1px solid rgba(247,147,26,0.3); }
.dot-gold { background: rgba(255, 215, 0, 0.15); color: #FFD700; border: 1px solid rgba(255,215,0,0.3); }
.dot-sp500 { background: rgba(0, 176, 255, 0.15); color: #00B0FF; border: 1px solid rgba(0,176,255,0.3); }
.dot-cash { background: rgba(16, 185, 129, 0.15); color: #10B981; border: 1px solid rgba(16,185,129,0.3); }

/* KPIs Grid */
.grid-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }
.kpi-card { background: var(--pod); border: 1px solid var(--line); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 4px; }
.kpi-title { font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--dim); letter-spacing: .05em; }
.kpi-value { font-size: 22px; font-weight: 900; color: var(--fg-hi); font-family: var(--mono); }
.kpi-sub { font-size: 11px; color: var(--mid); }

.grid-3col { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px; margin-top: 8px; }
.fitness-card { background: var(--inner); border: 1px solid var(--line); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 6px; }
.fitness-card h4 { font-size: 13px; font-weight: 800; color: var(--fg-hi); display: flex; justify-content: space-between; }
.fitness-card p { font-size: 11.5px; color: var(--mid); }
.fitness-weight { font-size: 11px; color: var(--sp500); font-weight: 700; }

.val-pos { color: var(--pos) !important; font-weight: 700; }
.val-neg { color: var(--neg) !important; font-weight: 700; }
.tag-badge { font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 4px; background: rgba(0,176,255,0.15); color: var(--sp500); border: 1px solid rgba(0,176,255,0.3); }
"#
}
