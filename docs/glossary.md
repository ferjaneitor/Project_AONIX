# Glosario AONIX

Términos centrales del proyecto. Cuando un documento usa uno de estos términos, debe entenderse según esta definición.

---

**Acción.** Unidad atómica de interacción del agente con AONIX. Pasa por el validador antes de ejecutarse. Ver [08](08-actions-and-rewards.md).

**Agente.** Entidad externa que propone acciones: humano, búsqueda exhaustiva, modelo de IA. No es AONIX.

**Aoncir** (`.aoncir`). AON Circuit Intermediate Representation. Documento técnico canónico del circuito, expandido a AND/OR/NOT. Ver [03](03-format-aoncir.md).

**Aonclg** (`.aonclg`). AON Canonical Learning Graph. Documento de contexto de aprendizaje asociado a un `.aoncir`. Nunca introduce primitivas. Ver [04](04-format-aonclg.md).

**AON.** AND, OR, NOT. Las tres primitivas únicas del sistema.

**AONIX.** AND-OR-NOT Integrated eXploration. El proyecto.

**Bus.** Etiqueta semántica que agrupa señales (no es una operación lógica nueva). Variantes: `main_bus`, `address_bus`, `data_bus`.

**Cono lógico.** Subconjunto del circuito que afecta una salida o región específica. Útil para simulación y validación incremental.

**Circuito.** Grafo dirigido (generalmente acíclico) cuyos nodos son AND, OR o NOT, con puertos de entrada y salida.

**Circuito compuesto.** Circuito de mayor nivel guardable como `.aoncir`, válido solo si internamente está expandido a AND/OR/NOT. No es una primitiva.

**Clock.** Etiqueta semántica que marca una señal como temporal en niveles donde aplica (≥ 11). No es una primitiva.

**Compuerta derivada.** XOR, XNOR, NAND, NOR, o cualquier función booleana expresable a partir de las primitivas. **No** es invocable como primitiva. Puede aparecer como comportamiento descubierto.

**Compuerta primitiva.** Únicamente AND, OR o NOT. Conjunto cerrado, no extensible.

**Coordinador central.** Módulo que orquesta el flujo de un episodio. No es una IA. Ver [10](10-coordinator.md).

**Currículo.** Sistema de niveles tipo videojuego que organiza el aprendizaje. Ver [06](06-curriculum.md).

**DAG.** Directed Acyclic Graph. Forma estándar del grafo de un circuito combinacional.

**Episodio.** Sesión de construcción de un circuito asociado a una tarea, con estado inicial, secuencia de acciones, y cierre con o sin promoción a memoria canónica.

**Etiqueta semántica.** Anotación sobre una señal o grupo (`carry`, `zero_flag`, `clock`, `bus`...) que informa al simulador, verificador, visualizador y traductor. No introduce primitivas.

**Evaluador.** Módulo que **mide** calidad estructural. No decide correctitud. Ver [02](02-architecture.md) y [07](07-testing-and-verification.md).

**Fan-in.** Número de entradas que llegan a un nodo.

**Fan-out.** Número de salidas que parten de un nodo (cuántos consumidores tiene una señal).

**Flag.** Salida con semántica especial (`carry`, `zero`, `overflow`, `negative`). Etiqueta, no primitiva.

**Hash canónico.** Huella estable de un circuito basada en su topología, puertos y etiquetas, calculada con ordenamiento determinista. Dos circuitos estructuralmente equivalentes tienen el mismo hash.

**Histórica (memoria).** Memoria append-only de versiones anteriores verificadas. Ver [05](05-memory-system.md).

**IA.** Agente externo opcional. No es fuente de verdad técnica. Ver [00 — Principio rector](00-vision.md).

**Mejora estricta.** Nueva versión que, según el ranking del evaluador, supera la oficial activa en todas las dimensiones críticas (o domina lexicográficamente bajo el orden activo). Ver [05](05-memory-system.md).

**Memoria canónica.** Memoria que guarda el `.aoncir` oficial activo por circuito y tamaño. Una sola versión activa por circuito.

**Memoria experimental.** Memoria de intentos no promovidos. Append-only.

**Memoria visual.** Memoria de layouts 2D. No afecta la verdad técnica.

**Mundo lógico.** Capa base de AONIX: tipos fundamentales (señales, compuertas, circuitos, tareas, pruebas). Ver [02](02-architecture.md).

**Nivel.** Peldaño curricular con tareas, pruebas y criterios de avance. Numerados 0 a 13 en alcance inicial.

**Oficial activo.** El único `.aoncir` vigente como verdad técnica de un circuito y tamaño. Reemplazable atómicamente.

**Optimización.** Transformación que mejora métricas estructurales preservando comportamiento. Ver [11 — Roadmap, Fase 6](11-roadmap.md).

**Optimizador.** Módulo que aplica transformaciones. Toda salida del optimizador se reverifica.

**Pruebas.** Conjuntos de casos que el verificador aplica. Pueden ser exhaustivas, aleatorias con semilla, por casos límite, por regresión, diferenciales, modulares, por propiedades. Ver [07](07-testing-and-verification.md).

**Puerto.** Entrada o salida externa de un circuito.

**Recompensa.** Señal de aprendizaje para agentes. No es fuente de verdad técnica. Jerarquía absoluta: correctitud > optimización > elegancia > velocidad. Ver [08](08-actions-and-rewards.md).

**Reemplazo atómico.** Promoción transaccional de una versión nueva a oficial activo: o se completa íntegra o no se completa.

**Regla absoluta.** Restricción inviolable del sistema. R1: mundo 2D. R2: primitivas solo AND/OR/NOT. Ver [01](01-rules-absolute.md).

**Regresión.** Versión nueva que falla pruebas que la oficial activa superaba. Se rechaza automáticamente.

**Reset.** Etiqueta semántica que vuelve un estado al inicial en niveles temporales.

**Semilla.** Valor que fija la secuencia de pruebas aleatorias. Garantiza reproducibilidad.

**Señal.** Nodo del grafo con un identificador único; representa un bit (0 o 1) en un punto del circuito.

**Señal muerta.** Señal no alcanzable desde ninguna salida del circuito. Penalizable.

**Simulador.** Módulo que **ejecuta** un circuito sobre una entrada de forma determinista. No decide correctitud.

**Tarea.** Unidad operativa: spec formal + nivel + restricciones + métricas. Define la meta del episodio. Ver [06](06-curriculum.md).

**Traductor humano.** Módulo que explica circuitos, errores, optimizaciones, decisiones a humanos en lenguaje natural derivado de estructura real. Ver [02 — capa 11](02-architecture.md).

**Traductor para IA.** Módulo que entrega estado y lista enumerable de acciones legales a un agente de IA. Ver [02 — capa 12](02-architecture.md).

**Trayectoria.** Secuencia de acciones de un episodio, con su retroalimentación paso a paso. Almacenada en memoria de trayectorias y en el `.aonclg`.

**Validador.** Módulo que decide si una acción es legal **antes** de ejecutarla. Filtro previo a simulador/verificador. Ver [02 — capa 4](02-architecture.md).

**Verificador.** Módulo que **decide** si un circuito cumple su especificación. Decisión binaria PASA/FALLA. Única fuente de la decisión de correctitud. Ver [07](07-testing-and-verification.md).

**Versión histórica.** `.aoncir` previamente oficial activo, ahora archivado en memoria histórica. Inmutable.

**Visualizador.** Capa Vulkan 2D que renderiza el mundo formal. No decide, no verifica, no altera. Ver [09](09-visualization-vulkan.md).

**Vulkan.** API gráfica 2D usada por AONIX. Backend exacto (`ash`, `wgpu`, `vulkano`) pendiente de decisión.
