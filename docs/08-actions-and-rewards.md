# 08 — Acciones y función de recompensa

## Acciones permitidas

Las acciones son la **única interfaz** entre un agente y AONIX. El validador las revisa antes de ejecutarlas; el simulador, verificador, evaluador y coordinador solo operan sobre acciones ya validadas.

### Acciones fundamentales sobre el circuito

Estas acciones operan **exclusivamente** sobre las primitivas permitidas:

- `create_gate_AND { id, inputs: [signal_id, signal_id], output: signal_id }` — aridad **estricta 2** en Fase 1.
- `create_gate_OR { id, inputs: [signal_id, signal_id], output: signal_id }` — aridad **estricta 2** en Fase 1.
- `create_gate_NOT { id, input: signal_id, output: signal_id }` — aridad estricta 1.
- `connect { from: signal_id, to: input_port_of_gate }` — conexión explícita si la representación lo requiere.
- `declare_signal { id, semantic_tag?, group? }` — declarar una señal interna.
- `assign_output { circuit_output: port_id, source: signal_id }` — atar una salida del circuito a una señal interna.
- `delete_dead_signal { id }` — eliminar señal no alcanzable desde ninguna salida.
- `delete_gate { id }` — eliminar una compuerta (válido solo si no rompe el grafo o si la rotura se compensa con otras acciones del mismo batch).

### Acciones de control e introspección

- `request_evaluation` — pide al evaluador métricas actuales del circuito parcial.
- `stop_construction` — el agente declara que ha terminado; el coordinador inicia verificación, evaluación y comparación con oficial activo.
- `propose_optimization { transformation_id }` — solicita aplicar una transformación catalogada del optimizador.
- `test_specific_input { input_vector }` — pide al simulador ejecutar un caso puntual.
- `request_visualization` — solicita refresco visual.
- `request_explanation { target: gate | signal | output }` — pide al traductor humano explicar una parte del circuito.

### Acciones prohibidas (rechazadas por el validador)

- `use_xor`, `use_xnor`, `use_nand`, `use_nor`, o cualquier variante que cree un nodo de tipo distinto a AND/OR/NOT. **Rechazo absoluto.**
- `import_gate { kind: derived }` — importar como primitiva una compuerta derivada. **Rechazo absoluto.**
- `declare_correct { circuit }` — un agente intentando declarar correcto su propio circuito sin pasar por el verificador. **Rechazo absoluto.**
- `bypass_validator`, `bypass_verifier`, `bypass_coordinator` — cualquier intento de saltarse módulos deterministas. **Rechazo absoluto.**
- `write_canonical_memory { circuit }` directo desde un agente sin promoción del coordinador. **Rechazo absoluto.**

### Sobre circuitos compuestos guardados

Los circuitos compuestos guardados en memoria canónica (full adders, ALUs, etc.) **no son compuertas primitivas**. Pueden:

- Consultarse.
- Compararse.
- Visualizarse como referencia.
- Estudiarse en su trayectoria de evolución.
- Servir como modelo de referencia para verificación de un circuito mayor que internamente los reimplementa.

Lo que **no** pueden hacer:

- Aparecer en un `.aoncir` final como nodos opacos sin expandir.
- Invocarse como acción `use_full_adder` que añade un nodo no-primitivo al grafo.
- Sustituir la obligación de expansión completa a AND/OR/NOT en la versión oficial activa.

(Conceptualmente: un circuito compuesto es una **definición canónica reutilizable**, no una **primitiva del mundo**. Su uso en composición está permitido, pero la composición se expande a primitivas en la representación final.)

## Diseño de la lista de acciones legales

En cada estado del episodio, el coordinador (vía validador) calcula la **lista exacta de acciones legales**. Esta lista:

- Es finita y enumerable.
- Excluye acciones imposibles en el estado actual (no se puede crear una compuerta sobre una señal inexistente).
- Excluye acciones prohibidas por el nivel (en el nivel 0, ninguna acción de construcción está permitida; solo introspección).
- Se entrega al traductor para IA como parte del estado.

La IA elige una acción **de esa lista**. Si elige fuera, el validador rechaza y emite retroalimentación.

## Validador (resumen formal)

Reglas que el validador aplica antes de aceptar una acción (ver [02 — Arquitectura](02-architecture.md) para detalle completo):

1. **Tipo permitido:** la compuerta es AND, OR o NOT. Cualquier otra cosa se rechaza.
2. **Señales definidas:** todas las señales referenciadas existen.
3. **Nombres únicos:** no se duplican IDs.
4. **Conexiones válidas:** aridad correcta, no hay self-loops directos.
5. **Sin ciclos no permitidos.**
6. **Salidas asignadas correctamente.**
7. **Acción compatible con el nivel.**
8. **Representación 2D respetada.**
9. **Sin escritura de memoria sin validación previa.**
10. **Sin declaración de correctitud unilateral.**

## Función de recompensa

La recompensa existe **para alimentar el aprendizaje futuro de IA**. AONIX no la usa para tomar decisiones técnicas (esas las toma el verificador/evaluador). Pero la recompensa es el canal de señal que un agente de aprendizaje recibe.

### Jerarquía absoluta de la recompensa

```
1. CORRECTITUD primero
2. OPTIMIZACIÓN después
3. ELEGANCIA después
4. VELOCIDAD después
```

**Un circuito incorrecto nunca puede superar a uno correcto por ser más pequeño.** La recompensa por correctitud domina las demás. Solo entre circuitos correctos se comparan las dimensiones inferiores.

Esto previene **reward hacking** estructural: un agente no puede ganar inflando la elegancia mientras sacrifica correctitud.

### Componentes positivos (recompensa premia)

- **Acciones válidas:** el validador acepta.
- **Reducción de errores:** disminución de casos fallidos respecto al estado anterior.
- **Mejoras estructurales:** disminución de compuertas, profundidad, señales muertas; aumento de reutilización.
- **Reutilización de señales:** compartir subexpresiones entre salidas.
- **Soluciones correctas:** el verificador entrega PASA.
- **Reducción de compuertas** sin perder correctitud.
- **Reducción de profundidad lógica.**
- **Superación de pruebas** por nivel.
- **Mejora frente a histórico:** la nueva versión bate las métricas de la oficial activa.
- **Estabilidad en pruebas aleatorias:** misma calidad con distintas semillas.
- **Resolución de casos límite** que la versión anterior fallaba.

### Componentes negativos (recompensa penaliza)

- **Acciones inválidas:** el validador rechaza.
- **Uso de compuertas prohibidas:** intento de introducir derivadas. **Penalización fuerte.**
- **Señales muertas:** subgrafo no alcanzable.
- **Compuertas redundantes:** equivalentes a la identidad o a constantes.
- **Salidas incorrectas** en cualquier caso de prueba.
- **Fallos en casos límite catalogados.**
- **Circuitos que solo aciertan parcialmente:** un agente no recibe crédito proporcional por estar "casi correcto".
- **Reward hacking:** patrones detectados de exploits (p.ej. construcciones que mejoran métricas sin pasar el verificador). **Penalización fuerte.**
- **Crecimiento innecesario:** añadir compuertas que no aportan.
- **Regresiones contra versiones anteriores.**

### Estructura de la recompensa

Recompensa total de un episodio:

```
R_total = α · R_correctitud
        + β · R_optimización
        + γ · R_elegancia
        + δ · R_velocidad
        − ε · R_penalización
```

Con la jerarquía absoluta forzada vía dominancia:

- `R_correctitud` es la componente principal. Si el circuito final no pasa el verificador, todas las demás se ponderan a cero o muy bajas para que `R_total ≪ R_total(circuito_correcto_subóptimo)`.
- Los coeficientes `α, β, γ, δ` se eligen tal que la diferencia entre "correcto" e "incorrecto" supere cualquier combinación posible de mejoras en las otras dimensiones.

### Recompensas intermedias (shaping)

Durante el episodio, el coordinador puede emitir recompensas parciales para guiar al agente:

- Pequeña recompensa positiva al producir una acción válida.
- Pequeña recompensa positiva al pasar pruebas parciales.
- Pequeña penalización al producir acción inválida.
- Penalización moderada al producir compuerta prohibida.

Las recompensas intermedias **no compensan el resultado final**. Acumular recompensas parciales no excede la recompensa de cierre por circuito correcto.

### Recompensa y memoria de aprendizaje

Cada acción del episodio, junto con su recompensa parcial, se registra en el `.aonclg`. La trayectoria completa permite reconstruir el aprendizaje (off-policy, replay, supervised distillation).

### Recompensa y currículo

El currículo no usa la recompensa directamente para decidir avance. Usa **criterios formales** (ver [06 — Currículo](06-curriculum.md)). La recompensa sirve al agente para aprender; el avance lo decide AONIX por demostración objetiva.

## Comprobación cruzada: la recompensa no es la verdad

Igual que con la IA: **la recompensa no es la fuente de verdad técnica**. La recompensa es una señal de aprendizaje. Si un agente acumula recompensa alta pero su circuito final no pasa el verificador, **el circuito final no entra en memoria canónica**, sin importar la recompensa.

La separación es estricta:

- **Verificador:** decide correctitud → escribe memoria canónica si gana.
- **Evaluador:** mide calidad → informa al verificador para comparación.
- **Recompensa:** señal de aprendizaje → escribe memoria de aprendizaje, no memoria canónica.

Esto previene cualquier circularidad: la recompensa no puede convertirse en la justificación de aceptar un circuito incorrecto.
