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

---

## Modelo visual 2D formal

> Esta sección define el **modelo visual normativo** de AONIX. Es el contrato que cumple cualquier implementación de la capa de visualización, independiente del backend Vulkan elegido.

### Espacio del modelo visual

El modelo visual vive en un espacio cartesiano 2D:

```
VisualSpace = ℝ²
```

Las posiciones son pares `(x, y)`. La unidad es abstracta (cell unit); la pantalla se proyecta con escalado en el momento del render. **No existe profundidad z** en la verdad técnica visual; cualquier ordenamiento Z entre capas es una preocupación del renderer, no del modelo.

### Elementos visuales primarios

```
VisualElement = OneOf {
    GateNode {
        gate_ref:    GateId         # referencia al nodo del .aoncir
        kind:        AND | OR | NOT
        position:    (x, y)
        size:        (w, h)         # implícito por kind si no se sobrescribe
        rotation:    0 | 90 | 180 | 270
        style:       StyleRef
        decoration:  [Decoration]   # halo, marca, badge
    }
  | PortNode {
        port_ref:    PortId
        role:        Input | Output
        position:    (x, y)
        semantic_tag: SemanticTag?
        group:       GroupId?
    }
  | SignalEdge {
        signal_ref:  SignalId
        path:        [(x, y)]       # polilínea 2D
        style:       StyleRef
        thickness:   Float
        annotations: [EdgeAnnotation]
    }
  | RegionBox {
        bbox:        ((x1,y1), (x2,y2))
        kind:        Bus | Cone | Block | Subcircuit | Highlight
        label:       String?
        opacity:     0..=1
        style:       StyleRef
    }
}
```

**Ningún elemento visual representa una primitiva distinta de AND, OR, NOT.** Si una visualización agrupa varias compuertas con un icono "estilo XOR", **el icono es una decoración de región**, no un GateNode. La verdad estructural permanece en el grafo subyacente y en el `.aoncir`.

### Reglas estructurales del modelo visual

1. **Cobertura completa.** Para todo nodo del grafo del `.aoncir` existe exactamente **un** `GateNode` en el modelo visual.
2. **Cobertura completa de puertos.** Para todo puerto del circuito existe exactamente **un** `PortNode`.
3. **Cobertura de señales.** Para toda señal existe **al menos un** `SignalEdge` (puede haber múltiples si la señal se ramifica visualmente; conceptualmente representan la misma señal).
4. **Sin nodos visuales sin grafo subyacente.** No se permite "añadir" un `GateNode` que no exista en el `.aoncir`. La visualización **espeja**, no inventa.
5. **Posiciones 2D válidas.** Toda posición pertenece al espacio 2D. No hay coordenadas no finitas, no hay `z`.
6. **Regiones no son operaciones.** Una `RegionBox` puede agrupar visualmente, pero no representa una compuerta nueva ni afecta la simulación.

### Capas (Z-order lógico)

El renderer organiza la composición visual en **capas lógicas**. El orden de las capas es fijo por especificación; la implementación física en Vulkan respeta este orden.

```
Z0  fondo
Z1  regiones (buses, bloques, subcircuitos como decoración)
Z2  aristas (signal edges)
Z3  nodos (gates + ports)
Z4  decoraciones de nodo (badges, marcas)
Z5  overlays de simulación (resaltado de flujo activo)
Z6  overlays de comparación (diff antes/después, regresión)
Z7  selección e interacción (cursores, halos de hover)
Z8  HUD (paneles laterales, métricas, controles)
```

Capas Z0–Z4 representan el **modelo del circuito**. Capas Z5–Z8 son **observación**.

### Estilo y semántica visual normativos

| Concepto | Convención visual normativa |
|---------|------------------------------|
| `AND` | Forma con lado plano y curva semicircular (convención clásica), color reservado A |
| `OR` | Forma con cola convexa y entrada cóncava (convención clásica), color reservado B |
| `NOT` | Triángulo con círculo de inversión, color reservado C |
| Puerto de entrada | Marcador a la izquierda del circuito, color D |
| Puerto de salida | Marcador a la derecha del circuito, color E |
| Señal activa = 1 | Color F (alto contraste con inactivo) |
| Señal activa = 0 | Color G |
| Señal no evaluada | Color H (atenuado) |
| Señal etiquetada `clock` | Patrón rítmico discreto o color reservado |
| Señal etiquetada `carry` | Color reservado para flags aritméticas |
| Señal etiquetada `bus` | Grosor de arista incrementado, etiqueta con `width` |
| Camino crítico | Color de acento, grosor incrementado |
| Compuerta redundante detectada | Halo de advertencia |
| Señal muerta | Atenuada al 30% de opacidad |
| Diferencia entre esperado y producido | Halo rojo en el nodo discrepante |

Los colores concretos los elige la paleta activa (configurable; al menos una paleta accesible para daltonismo). La **identidad conceptual** del color (qué representa) es normativa.

### Geometría de las aristas

Las aristas usan **enrutamiento ortogonal por defecto**: segmentos horizontales y verticales. Las variantes diagonales son aceptables en modos específicos (layout force-directed, p. ej.) pero el enrutamiento ortogonal es el canónico.

Reglas:

- Sin solapamiento de aristas a menos que la densidad lo obligue.
- Cruces marcados con "puente" (curva pequeña en una de las dos aristas).
- Bus etiquetado dibujado como conjunto de aristas paralelas con marca de ancho.

### Determinismo del layout

Para una estrategia de layout dada y un `.aoncir` dado, el resultado visual es **determinista**. Esto permite:

- Snapshot reproducible.
- Tests visuales por diferencia de imagen (con tolerancia).
- Comparación frame a frame entre versiones.

Excepción: el layout `force-directed` puede usar inicialización aleatoria; en ese caso requiere **semilla explícita** y se registra para reproducibilidad.

### Anclaje al modelo lógico

Cada `VisualElement` mantiene un puntero al elemento del grafo que representa (`gate_ref`, `port_ref`, `signal_ref`). Eliminar un elemento del grafo elimina su representación visual; añadir uno al grafo lo añade. **El modelo visual es siempre coherente con el modelo lógico**, garantizado por construcción.

### Estados de la visualización

El visualizador puede estar en uno de los siguientes estados (no excluyentes en su mayoría):

- **Static**: render del circuito sin animación.
- **Simulating**: una entrada propagándose; señales se actualizan secuencialmente.
- **Comparing**: dos circuitos lado a lado o superpuestos.
- **Replaying**: trayectoria del `.aonclg` paso a paso.
- **Highlighting**: cono lógico, camino crítico, región seleccionada destacada.
- **DiffMode**: cambios respecto a versión anterior (regiones añadidas/eliminadas/modificadas).
- **InteractiveBuild**: el agente añade/quita elementos (con validación por acción legal).

### Modelo de interacción

Cada interacción del usuario es:

1. **Visual-side intent** (clic, hover, drag).
2. **Mapped to a logical query** (¿qué nodo?, ¿qué cono?, ¿qué acción legal?).
3. **Routed to the appropriate module** (validador para construcción, simulador para test, traductor para explicación).
4. **Result rendered as updated visual state**.

Las interacciones **no** modifican directamente el `.aoncir`. Cualquier modificación pasa por una acción formal validada (ver [08](08-actions-and-rewards.md)).

### Garantías del modelo visual

1. **Cobertura total**: todo lo del modelo lógico se puede ver.
2. **Fidelidad estructural**: lo que se ve corresponde a la verdad técnica.
3. **No invención**: el visualizador no añade nodos lógicos.
4. **Sin invención de primitivas**: las decoraciones no representan compuertas nuevas.
5. **Determinismo**: misma entrada visual + mismo modelo lógico ⇒ misma imagen (módulo tolerancia de render).
6. **Reversibilidad**: cualquier vista puede recalcularse desde el `.aoncir`; el modelo visual es derivado.

### Lo que el modelo visual **no** puede hacer

- Mostrar un nodo de tipo distinto de AND/OR/NOT como si fuera una primitiva.
- Ocultar permanentemente compuertas reales (puede colapsar regiones, pero el desglose siempre está disponible).
- Diferir de la verdad técnica.
- Funcionar como fuente para reconstruir el `.aoncir` (es derivado, no fuente).
- Decidir corrección, métricas o promoción.

### Decisión pendiente

El **backend gráfico** concreto (`ash`, `wgpu`, `vulkano`) sigue como decisión documentada pero no fijada. Ver [11 — Roadmap, decisiones pendientes](11-roadmap.md). El modelo visual normativo definido aquí es **independiente del backend**.
