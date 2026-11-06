pub fn render_head_to_head_section() -> &'static str {
    r#"
<section id="mano-a-mano">
  <div class="section-head">
    <div>
      <h2>Comparador Mano a Mano de Estrategias (2 o 3 Candidatos)</h2>
      <p class="sub">Confrontación directa y punto a punto entre 2 o 3 estrategias simultáneas. Audita diferencias porcentuales, control de caídas, eficiencia de capital y consistencia matemática.</p>
    </div>
  </div>

  <div class="pod">
    <!-- Barra de Selección de Candidatos Mano a Mano -->
    <div class="ctrl-bar h2h-ctrl-bar">
      <div class="h2h-select-grp">
        <label for="select-h2h-a" class="h2h-lbl"><span class="h2h-tag tag-a">Candidato A</span></label>
        <select id="select-h2h-a" class="select-box h2h-select">
          <!-- Opciones pobladas dinámicamente -->
        </select>
      </div>

      <button type="button" id="btn-h2h-swap" class="btn-swap" title="Rotar Selección de Candidatos">
        ⇄ Rotar
      </button>

      <div class="h2h-select-grp">
        <label for="select-h2h-b" class="h2h-lbl"><span class="h2h-tag tag-b">Candidato B</span></label>
        <select id="select-h2h-b" class="select-box h2h-select">
          <!-- Opciones pobladas dinámicamente -->
        </select>
      </div>

      <div class="h2h-select-grp">
        <label for="select-h2h-c" class="h2h-lbl"><span class="h2h-tag tag-c">Candidato C</span></label>
        <select id="select-h2h-c" class="select-box h2h-select">
          <option value="none">(Ninguno — Modo 2 Estrategias)</option>
          <!-- Opciones pobladas dinámicamente -->
        </select>
      </div>
    </div>

    <!-- Tarjetas Resumen A vs B vs C -->
    <div class="h2h-cards-grid" id="h2h-cards-container">
      <div class="h2h-card card-a" id="h2h-summary-a">
        <!-- Contenido generado por JS -->
      </div>
      <div class="h2h-card card-b" id="h2h-summary-b">
        <!-- Contenido generado por JS -->
      </div>
      <div class="h2h-card card-c" id="h2h-summary-c" style="display: none;">
        <!-- Contenido generado por JS -->
      </div>
    </div>

    <!-- Gráfico Comparativo Visual de KPIs Clave -->
    <div class="chart-container" style="margin-top: 14px;">
      <div class="panel-header" style="margin-bottom: 8px;">
        <div class="panel-info">
          <h3>Comparativa Visual Relativa de Indicadores Clave</h3>
        </div>
      </div>
      <div style="position: relative; height: 320px;">
        <canvas id="chart-h2h-bar"></canvas>
      </div>
    </div>

    <!-- Tabla Detallada de Métricas Mano a Mano -->
    <div class="panel-header" style="margin-top: 18px; margin-bottom: 8px;">
      <div class="panel-info">
        <h3>Matriz Detallada de Métricas Cuantitativas</h3>
        <p>Valores absolutos y cálculo dinámico de ventaja relativa y liderazgo cuantitativo.</p>
      </div>
    </div>
    <div class="tabla-wrap">
      <table class="tabla tabla-h2h">
        <thead id="h2h-thead">
          <tr id="h2h-thead-tr">
            <th class="al-l" style="width: 28%;">Métrica Cuantitativa</th>
            <th class="al-r" id="h2h-th-a" style="width: 26%;">Candidato A</th>
            <th class="al-c" id="h2h-th-mid" style="width: 20%;">Ventaja Relativa</th>
            <th class="al-r" id="h2h-th-b" style="width: 26%;">Candidato B</th>
          </tr>
        </thead>
        <tbody id="h2h-table-body">
          <!-- Filas generadas dinámicamente por JS -->
        </tbody>
      </table>
    </div>
  </div>
</section>
"#
}
