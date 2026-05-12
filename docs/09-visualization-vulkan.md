# 09 — Visualización 2D con Vulkan

## Principio

La visualización es **observación, no autoridad**.

> Vulkan no decide, no verifica y no altera el circuito. Solo visualiza.

La capa de visualización 2D existe para que humanos y herramientas externas puedan ver lo que el mundo formal contiene. **No tiene voz** sobre lo que es correcto, óptimo, aceptable o canónico.

## Por qué Vulkan

- **Rendimiento:** la visualización debe soportar grafos grandes (miles de nodos) con interacción fluida.
- **Determinismo controlable:** el pipeline gráfico es explícito y predecible.
- **Portabilidad:** Vulkan está disponible en Windows, Linux y macOS (vía MoltenVK).
- **Bajo nivel:** AONIX no necesita las abstracciones de un motor de juegos; necesita control sobre buffers, pipelines y comandos.

## Qué se visualiza

### Estructura del circuito

- **Grafo 2D** del circuito completo o de subregiones.
- **Compuertas** AND, OR, NOT con formas y colores distintos (consistencia visual estable).
- **Señales** como aristas dirigidas; grosor o color modulables por etiqueta semántica.
- **Entradas y salidas** del circuito como puertos diferenciados.
- **Buses** como conjuntos de señales agrupados visualmente.
- **Flags** como salidas etiquetadas con icono o color reservado por tipo (`carry`, `zero`, etc.).
- **Conos lógicos** resaltables por salida.
- **Rutas críticas** destacadas sobre el grafo base.
- **Regiones eliminadas** por optimización (vista comparativa antes/después).
- **Profundidad lógica** representada por niveles (eje vertical) o por gradiente de color.

### Dinámica de simulación

- **Flujo de señal por entrada** — animación o resaltado que muestra cómo se propaga una entrada concreta a través del circuito.
- **Salida esperada vs producida** — comparación lado a lado.
- **Diferencias** marcadas explícitamente.
- **Errores** en la salida resaltados (nodo donde la propagación produce el bit erróneo).

### Estado del episodio

- **Tarea actual** y nivel.
- **Progreso curricular** del agente.
- **Memoria activa** (qué `.aoncir` están vigentes, qué versiones históricas relevantes existen).
- **Pruebas superadas** y **pruebas fallidas** con su categoría.
- **Trayectoria de construcción** — acción a acción, paso a paso, replay.

### Cambios y evolución

- **Antes/después** de una optimización.
- **Versiones históricas** comparadas en una vista lineal o de árbol.
- **Reemplazos atómicos** de versión oficial activa con animación de transición que deja claro qué cambió.

## Lo que la visualización **no** hace

- **No modifica el `.aoncir`.** Cambiar el layout en pantalla no cambia la verdad técnica. El layout vive en la **memoria visual** (ver [05 — Memorias](05-memory-system.md)), separada de la memoria canónica.
- **No declara correcto un circuito.** Resaltar visualmente que la salida coincide con la esperada no equivale a verificación.
- **No oculta complejidad.** Si el circuito es feo, la visualización debe mostrarlo feo. No hay "modo elegancia" que esconda compuertas reales.
- **No inventa primitivas.** Aunque visualmente se pudiera agrupar varias compuertas en un icono de "XOR", el agrupamiento es solo etiqueta visual. El grafo subyacente sigue siendo AND/OR/NOT, y el `.aoncir` se exporta sin esa abstracción.

## Layout 2D

### Estrategias de layout

AONIX precisará al menos:

- **Layered** (por profundidad lógica) — útil para ver propagación.
- **Force-directed** — útil para grafos densos sin estructura jerárquica clara.
- **Manual** — el usuario fija posiciones; se guardan en memoria visual.
- **Bus-aware** — agrupa señales etiquetadas como bus en líneas paralelas; reduce visual clutter.
- **Cone-focused** — destaca un solo cono lógico, colapsa lo demás.

El layout activo se elige por contexto (tarea, nivel, modo de exploración).

### Determinismo del layout

El auto-layout debe ser **determinista** dado un circuito y una estrategia. Mismo circuito + misma estrategia ⇒ mismo layout (no hay rendering aleatorio que cambie entre sesiones).

### Persistencia

Layouts curados manualmente se guardan en memoria visual y se asocian al hash canónico del circuito (no al nombre, para sobrevivir a renombramientos).

## Interacciones

El visualizador soporta:

- **Zoom y pan** (navegación 2D).
- **Selección** de nodo o señal (muestra detalles en panel lateral).
- **Resaltar cono** desde una salida.
- **Trazar flujo** desde una entrada hasta una salida.
- **Ejecutar entrada específica** y ver el flujo animado (interacción con simulador).
- **Comparar versiones** (split view o overlay).
- **Replay de trayectoria** desde memoria de trayectorias.
- **Inspeccionar acción** sobre un nodo (qué acción del agente lo creó, cuándo, con qué resultado del validador).

Las interacciones **no modifican** el circuito directamente; cualquier modificación pasa por acciones formales que el validador revisa.

## Modos de visualización

| Modo | Propósito |
|------|-----------|
| Construcción | Vista activa durante un episodio; muestra circuito parcial y acciones legales |
| Verificación | Resalta cobertura de pruebas, casos fallidos, propiedades violadas |
| Optimización | Antes/después; señales eliminadas; compuertas reusadas |
| Comparación | Dos `.aoncir` lado a lado (versión nueva vs oficial activa) |
| Evolución | Línea de tiempo de versiones históricas |
| Curricular | Mapa del progreso por nivel |
| Memoria | Navegador de circuitos guardados |

Cambiar de modo no altera nada del mundo formal; es solo perspectiva.

## Render: estilo

Recomendación inicial (no normativa, ajustable):

- **Fondo:** neutro, claro u oscuro elegible.
- **AND:** forma con curva semicircular y lado plano (convención clásica), color reservado.
- **OR:** forma con cola convexa, color reservado.
- **NOT:** triángulo con círculo de inversión, color reservado.
- **Señales:** aristas con grosor que indica fan-out, color que indica:
  - Por estado: activa / inactiva / no evaluada.
  - Por etiqueta: `clock`, `carry`, `bus` con paletas diferenciadas.
- **Conos lógicos:** sombreado de región semitransparente.
- **Camino crítico:** resaltado en color de acento.
- **Compuertas redundantes detectadas:** marca de advertencia.
- **Señales muertas:** atenuadas o marcadas.

La paleta y formas deben ser **consistentes** y elegidas para alta legibilidad (consideración de daltonismo).

## Rendimiento

Para un circuito de 10⁴ nodos:

- Frame target: ≥ 60 FPS en navegación.
- Layout completo recalculado: ≤ 1 s.
- Simulación animada: paso a paso fluido.

Para 10⁵+ nodos:

- Niveles de detalle (LOD): clustering visual de subgrafos lejanos.
- Streaming de regiones.
- Indexación espacial.

## Arquitectura de la capa

```
aonix-vis crate
├── render/         # Pipeline Vulkan, swapchain, command buffers
├── layout/         # Algoritmos de posicionamiento
├── style/          # Paletas, formas, tipografía
├── interaction/    # Input, selección, picking
├── overlays/       # Conos, rutas, flujos, comparaciones
└── api/            # Interfaz para consumidores (CLI, GUI futura)
```

### Dependencias previstas en Rust

Decisión pendiente entre:

- **`ash`** — bindings directos a Vulkan; control total, más boilerplate.
- **`wgpu`** con backend Vulkan — abstracción WebGPU encima, más portable, menos control fino.
- **`vulkano`** — wrapper de seguridad sobre Vulkan en Rust idiomático.

**Recomendación inicial:** evaluar `ash` para máximo control (alineado con la filosofía determinista de AONIX) o `wgpu` si la portabilidad multiplataforma uniforme es prioritaria. Decisión a confirmar con el usuario.

## Separación estricta de roles

| Capa | Lee | Escribe |
|------|-----|---------|
| Visualización | `.aoncir` activos, memoria visual, estado de simulación, decisiones del verificador, métricas del evaluador | **Solo** memoria visual (layouts) |
| Visualización | **Nunca** escribe en | Memoria canónica, histórica, de aprendizaje, de pruebas, curricular |

Esta separación garantiza que **borrar todos los layouts no afecta la verdad técnica del proyecto**.

## Casos de uso futuros

- Exportación a SVG/PDF para artículos académicos.
- Captura de "highlight reels" de un agente aprendiendo (replays editables del `.aonclg`).
- Modo presentación para docencia.
- Dashboard de seguimiento de progreso curricular agregado.
