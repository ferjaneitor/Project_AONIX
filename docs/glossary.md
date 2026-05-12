# Glosario AONIX

> **Documento normativo.** Las definiciones aquí establecidas son **autoritativas**. Cuando cualquier otro documento del proyecto usa uno de estos términos debe entenderse exactamente según esta definición. Cuando un documento parezca contradecir el glosario, el glosario manda; el otro documento se corrige.
>
> El glosario se amplía añadiendo entradas. Modificar una entrada existente requiere auditoría: el término puede haber sido referenciado desde el código, los tests o los `.aoncir`/`.aonclg` ya producidos.

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

---

## Términos adicionales (consolidación normativa)

**AgentVisibilitySet.** Conjunto enumerable de fuentes de información que un agente puede leer durante un episodio. Definido por la tarea y modulable por modos del currículo. Ver [16](16-ai-visibility-limits.md).

**Atajo (operativo).** Información que, entregada a un agente durante un episodio activo, le permitiría producir la solución sin construirla. Prohibido. Ver [16](16-ai-visibility-limits.md), [18](18-operational-vs-non-operational-memory.md).

**Bloqueante (caso límite).** Caso límite cuya superación es condición necesaria para que el verificador entregue PASA. Si falla, el circuito se rechaza. Ver [07](07-testing-and-verification.md).

**Capa de aceptación.** Una de las dos puertas formales (estructural y funcional+calidad) por las que pasa un circuito antes de ser aceptado. Ver [13](13-circuit-acceptance.md).

**Capa de rechazo (L0–L5).** Niveles de rechazo formal en AONIX, desde acción individual rechazada por validador hasta tarea rechazada por cargador. Ver [14](14-circuit-rejection.md).

**Catálogo de tareas.** Conjunto versionado de declaraciones formales de tareas disponibles en AONIX. Ver [12](12-task-specification.md).

**Catálogo de transformaciones.** Conjunto de transformaciones del optimizador, con precondición, efecto y garantía de preservación de comportamiento. Ver [15](15-optimization-rules.md).

**Degradado (oficial activo).** Estado de un `.aoncir` oficial activo que falla una re-verificación con suite ampliada. Inicia proceso de revisión humana. Ver [19](19-versioning-policy.md).

**Diff visual (modo).** Estado del visualizador que muestra diferencias entre dos versiones (típicamente antes/después de una optimización o versión histórica vs oficial activa). Ver [09](09-visualization-vulkan.md).

**Episodio cerrado.** Episodio cuya secuencia de acciones terminó y cuyo `.aonclg` quedó escrito de forma inmutable. Ver [10](10-coordinator.md), [17](17-aoncir-aonclg-relationship.md).

**Estado parcial.** Estado intermedio del circuito durante la construcción de un episodio. No es una versión canónica ni histórica. Puede serializarse para snapshots. Ver [12](12-task-specification.md).

**Familia (de circuitos).** Conjunto de circuitos compuestos que comparten nombre y semántica pero difieren en parámetros (típicamente `width`). Comparten suites de regresión y modelos de referencia. Ver [19](19-versioning-policy.md).

**Filtración (test de).** Test del propio AONIX que verifica que ningún canal lateral entrega información no operativa al agente. Ver [16](16-ai-visibility-limits.md).

**Huérfano (`.aonclg`).** `.aonclg` cuyo `.aoncir` referenciado ha sido eliminado por operación administrativa. Se marca como huérfano y se conserva. Ver [17](17-aoncir-aonclg-relationship.md).

**Identidad estructural.** Equivalencia entre dos circuitos basada en el `hash_canonical`. Dos circuitos con el mismo hash son **la misma versión** desde el punto de vista de memoria canónica. Ver [03](03-format-aoncir.md), [19](19-versioning-policy.md).

**Inmutabilidad de cierre.** Propiedad por la cual un `.aonclg` ya cerrado **no puede modificarse**. Garantía de auditoría histórica. Ver [04](04-format-aonclg.md), [17](17-aoncir-aonclg-relationship.md).

**Memoria operativa.** Memoria cuyo contenido se entrega al agente durante un episodio activo. Ver [18](18-operational-vs-non-operational-memory.md).

**Memoria no operativa.** Memoria que existe, se conserva y se audita, pero **no se entrega** al agente durante un episodio activo. Ver [18](18-operational-vs-non-operational-memory.md).

**Modo guided-onboarding.** Modo de currículo en niveles iniciales que permite mostrar fragmentos de soluciones modelo como ilustración pedagógica, sin convertirlos en compuertas reutilizables. Ver [16](16-ai-visibility-limits.md).

**Modo post-mortem.** Modo posterior al cierre de un episodio fallido que entrega información adicional al agente sobre causas de fallo, sin entregar la solución. Ver [16](16-ai-visibility-limits.md).

**Modo review-after-solve.** Modo posterior al cierre exitoso de un episodio que permite al agente comparar su solución con el oficial activo. Ver [16](16-ai-visibility-limits.md).

**Modo study-historical.** Modo que permite a un agente examinar versiones históricas para análisis, fuera de episodios activos sobre la misma tarea. Ver [16](16-ai-visibility-limits.md).

**Modelo visual normativo.** Especificación independiente del backend gráfico del modelo 2D de elementos visuales y sus reglas. Ver [09](09-visualization-vulkan.md) § Modelo visual 2D formal.

**Promoción atómica.** Transacción transaccional que mueve una versión nueva a oficial activa y archiva la incumbente. O ocurre íntegra, o no ocurre. Ver [19](19-versioning-policy.md).

**Puerta de aceptación.** Conjunto de condiciones formales que un circuito debe satisfacer en un punto del pipeline. AONIX tiene puerta estructural, puerta funcional+calidad y puerta de promoción. Ver [13](13-circuit-acceptance.md).

**Ranking order.** Orden lexicográfico declarado por una tarea sobre las métricas del evaluador, usado para decidir si una versión mejora estrictamente a otra. Ver [13](13-circuit-acceptance.md), [19](19-versioning-policy.md).

**Reward hacking.** Patrón de comportamiento de un agente que maximiza recompensa sin cumplir la meta real (p.ej. inflando métricas sin pasar el verificador). AONIX lo cierra por construcción mediante validador, verificador y separación de roles. Ver [08](08-actions-and-rewards.md), [16](16-ai-visibility-limits.md).

**Solución aceptada.** Circuito que pasa puerta estructural y puerta funcional+calidad. No implica promoción; puede quedar como solución no promovida si no mejora al oficial activo. Ver [13](13-circuit-acceptance.md).

**Solución promovida.** Solución aceptada que además mejora estrictamente al oficial activo según el `ranking_order` y supera la puerta de promoción del coordinador. Reemplaza al oficial activo. Ver [13](13-circuit-acceptance.md), [19](19-versioning-policy.md).

**Specification (Spec).** Descripción formal del comportamiento esperado de un circuito. Puede ser tabla de verdad, lista de propiedades, función de referencia, circuito de referencia o spec temporal. Ver [12](12-task-specification.md).

**Suite de regresión.** Conjunto acumulativo append-only de casos de prueba que en algún momento detectaron un fallo. Crece con el sistema; nunca se reduce sin justificación auditable. Ver [07](07-testing-and-verification.md).

**TemporalSpec.** Especificación de comportamiento de tareas temporales, evaluada sobre secuencias de ciclos en lugar de vectores únicos. Aplica a niveles ≥ 11. Ver [12](12-task-specification.md).

**Transformación legítima.** Transformación del catálogo del optimizador con garantía de preservación de comportamiento por vía algebraica y vía verificación. Ver [15](15-optimization-rules.md).

**Veredicto del verificador.** Decisión binaria PASA o FALLA emitida por el verificador sobre un circuito con respecto a una suite. Único determinante de la corrección. Ver [07](07-testing-and-verification.md).

**Versión experimental.** Versión verificada pero no promovida (por L2 o L4). Vive en memoria experimental. Ver [14](14-circuit-rejection.md), [19](19-versioning-policy.md).

**Withdrawn (versión).** Marca aplicada a una versión que fue oficial activa pero se reveló defectuosa (bug del propio AONIX). No se borra; se desactiva con causa. Ver [19](19-versioning-policy.md).
