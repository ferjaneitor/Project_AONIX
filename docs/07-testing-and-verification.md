# 07 — Pruebas y verificación

## Tres conceptos distintos

AONIX distingue tres conceptos que con frecuencia se confunden:

| Concepto | Pregunta que responde | Quién decide |
|----------|-----------------------|--------------|
| **Simulación** | ¿Qué hace este circuito en esta entrada? | Simulador (calcula) |
| **Verificación** | ¿Este circuito cumple su especificación? | Verificador (decide) |
| **Evaluación** | ¿Cómo de buena es la estructura de este circuito? | Evaluador (mide, no decide) |

Este documento cubre **verificación** y el **sistema de pruebas escalables** que la alimenta. El simulador y el evaluador se mencionan donde sea necesario.

## Determinismo total

La simulación es determinista por construcción.

**Garantía:** misma entrada + mismo circuito ⇒ mismo resultado. Siempre. En cualquier máquina. En cualquier momento.

Esto implica:

- Orden de evaluación reproducible (ordenamiento topológico determinista).
- Sin estado oculto.
- Sin paralelismo no determinista en la evaluación lógica.
- Sin floats (todo es booleano).
- Las pruebas aleatorias usan **semilla explícita**; misma semilla ⇒ misma secuencia.

## Sistema de pruebas escalables

**Regla:** cuanto más alto el nivel, más capas de prueba debe superar el circuito.

### Lógica pequeña: exhaustiva completa

Mientras el espacio de entradas sea viable, **prueba todas las combinaciones**.

| Entradas | Combinaciones |
|----------|---------------|
| 1 | 2 |
| 2 | 4 |
| 3 | 8 |
| 4 | 16 |
| 8 | 256 |
| 12 | 4 096 |
| 16 | 65 536 |
| 20 | ~1 M (límite blando) |

El umbral de viabilidad es configurable. Por encima del umbral, exhaustiva deja de ser práctica y se complementa.

### Circuitos medianos: combinación

Capas que se combinan:

1. **Pruebas exhaustivas por submódulo.** Si el circuito se descompone en bloques pequeños, cada bloque se verifica exhaustivamente.
2. **Pruebas aleatorias reproducibles** con semilla fija (p.ej. 10 000 vectores).
3. **Casos límite catalogados:**
   - Todo-cero.
   - Todo-uno.
   - Patrones alternados (010101..., 101010...).
   - Valores mínimos (-MAX, 0, +MAX según interpretación).
   - Valores máximos.
   - Un solo bit activo.
   - Un solo bit apagado.
   - Casos históricamente fallidos (acumulados en suite de regresión).
4. **Suite de regresión** acumulada: todo caso que alguna vez detectó un bug se queda.

### Circuitos grandes: estrategias amplias

5. **Simulación dirigida** — el verificador genera entradas que estresan caminos específicos.
6. **Verificación por propiedades** — propiedades lógicas formales (p.ej. "para todo A, sum(A, 0) = A").
7. **Validación por conos lógicos** — verificar salidas una por una usando solo su cono.
8. **Comparación contra modelo de referencia** — un modelo aritmético en software o un circuito conocido bueno.
9. **Regresiones automáticas.**
10. **Pruebas diferenciales** — dos circuitos distintos que deberían comportarse igual se ejecutan en paralelo y se comparan.
11. **Pruebas por bloques** — descomposición jerárquica.
12. **Verificación modular** — cada subcircuito verificado por separado y luego compuesto.
13. **Validación incremental** — tras un cambio, recalcular solo lo afectado.

### Circuitos arquitectónicos: temporales

14. **Secuencias temporales** — programas de N ciclos.
15. **Ciclos de reloj** modelados explícitamente.
16. **Pruebas de reset** — el sistema vuelve al estado correcto.
17. **Pruebas de enable** — la señal `enable` controla efectivamente.
18. **Propagación de carry** — verificada a lo largo del datapath.
19. **Validación de flags** — `zero`, `carry`, `overflow`, `negative` correctos en cada estado.
20. **Programas pequeños** — 3 a 20 instrucciones; verificación post-ejecución de registros y memoria.
21. **Escenarios de memoria** — lecturas, escrituras, colisiones, direcciones extremas.
22. **Secuencias largas reproducibles** — con semilla fija.
23. **Pruebas de regresión arquitectónica.**

## Validación por entrada específica

AONIX debe poder ejecutar **una sola entrada** de forma rápida. Esto habilita interacción tipo videojuego.

Para una entrada dada, debe mostrar:

- Entrada actual.
- Salida esperada (si la tarea tiene spec).
- Salida producida.
- Señales internas activadas.
- Ruta lógica usada (qué nodos dispararon).
- Compuertas involucradas.
- Diferencia entre esperado y producido.

Esta capacidad es esencial para la visualización 2D (resaltar el flujo) y para el traductor humano (explicar el porqué).

## Validación incremental

Cuando una acción modifica una parte del circuito, AONIX **no recalcula todo**.

**Conos lógicos:** subconjunto del circuito que afecta una salida o región. Cada salida tiene su cono; cada cambio afecta a un conjunto de conos.

Flujo:

1. Acción cambia una compuerta o conexión.
2. Validador determina qué señales corriente abajo se afectan.
3. Verificador re-evalúa solo los conos que tocan esas señales.
4. Evaluador recalcula métricas locales y propaga al agregado.

Beneficios:

- Acelera simulación durante construcción interactiva.
- Acelera validación durante optimización.
- Acelera entrenamiento de IA (más episodios por unidad de tiempo).
- Permite mostrar cambios locales (la visualización resalta solo la región afectada).
- Permite optimización por región sin tocar el resto.

## Casos límite catalogados

AONIX mantiene un catálogo formal de casos límite, organizado por:

- **Por aridad:** todo-cero, todo-uno, bit único activo, bit único apagado.
- **Por patrón:** alternados, secuencias de Gray, patrones de transición.
- **Por interpretación:** valores mínimos/máximos en lectura signo-magnitud, complemento a uno, complemento a dos.
- **Por flag:** entradas que disparan exactamente `overflow`, `zero`, `negative`, `carry`.
- **Por histórico:** todo caso que alguna vez detectó un bug en cualquier circuito permanece en el catálogo para futuros circuitos comparables.

El catálogo crece con el uso. Nunca se reduce.

## Pruebas por propiedades

Una **propiedad** es un enunciado universal que el circuito debe cumplir para toda entrada (o para toda entrada de una clase).

Ejemplos:

- **Conmutatividad:** `sum(A, B) == sum(B, A)` para sumador.
- **Asociatividad lógica:** `(A AND B) AND C == A AND (B AND C)`.
- **Identidad:** `A OR 0 == A`, `A AND 1 == A`.
- **Inverso temporal:** tras `reset`, el estado vuelve al inicial.
- **Idempotencia de write/read:** `read(addr, write(addr, v, mem)) == v`.

Las propiedades se prueban con muestreo aleatorio masivo o con SMT/SAT cuando el espacio es tratable.

## Comparación contra modelo de referencia

Un modelo de referencia es:

- Un programa software equivalente (sumador aritmético en `u32` para comparar contra `thirty_two_bit_full_adder`).
- Un circuito anteriormente verificado tomado como ground truth.
- Una especificación formal evaluable.

El verificador ejecuta el circuito y el modelo sobre el mismo conjunto de entradas y compara salidas y flags.

## Verificación modular

Cuando un circuito grande se compone de subcircuitos previamente verificados, el verificador puede:

1. Verificar cada subcircuito por separado (rápido).
2. Verificar la composición (interconexión correcta, sin acoplos no previstos).
3. Verificar el comportamiento global solo sobre casos límite y aleatorios (no es necesario exhaustiva si los subcircuitos lo son).

Esto permite escalar.

## Regresión automática

Cada vez que un caso fallido se detecta y se corrige, ese caso se añade a la suite de regresión del circuito **y de la familia** (p.ej. todos los sumadores comparten una suite de regresión común para casos universales).

La suite de regresión nunca se reduce sin justificación auditable.

## Decisión binaria del verificador

El verificador entrega, al final, una decisión **binaria por suite**:

```
verificacion(circuito, nivel) -> {
    decision: PASA | FALLA,
    suite_id: ...,
    casos_evaluados: N,
    casos_fallidos: [ ... ],
    seed: ...,
    tiempo: ...,
    detalle_por_caso: ...,
    propiedades_violadas: [ ... ]
}
```

No hay decisiones difusas. No hay "pasó casi". Pasa o no pasa.

(Las métricas estructurales — esas sí son continuas — las da el evaluador, no el verificador.)

## Política sobre falsos negativos

Si una versión nueva del circuito **falla una prueba que la oficial activa supera**, eso es **regresión** y la versión nueva se rechaza, aunque sus métricas estructurales sean mejores.

La correctitud absoluta tiene prioridad sobre la optimización.

## Política sobre falsos positivos

Si una versión pasa todas las suites del nivel pero falla casos límite que el catálogo aún no había incorporado, y esos casos se descubren después, AONIX:

1. Añade los casos al catálogo y a la suite del nivel.
2. Marca la versión como **provisionalmente oficial** (sello degradado) o la retira si los casos fallidos son críticos.
3. Registra la lección en memoria de fallos.
4. Reabre la tarea para revisión.

Esto es operación normal. AONIX aprende a probar mejor con el tiempo.

## Tiempo de prueba y presupuesto

Cada nivel tiene un **presupuesto de tiempo de verificación** orientativo (no estricto). Niveles bajos completan en milisegundos; niveles arquitectónicos pueden tomar segundos a minutos. El sistema reporta el tiempo en el `.aonclg` y en el reporte de verificación del `.aoncir`.

## Reproducibilidad total

Toda ejecución de prueba registra:

- Semilla.
- Versión del verificador.
- Versión del modelo de referencia.
- Versión del `.aoncir`.
- Hash de la suite usada.

Cualquiera puede reproducir el resultado bit a bit.

---

## Matriz normativa de pruebas por nivel

> Esta matriz consolida, por nivel curricular, **qué tipos de prueba son obligatorios**, **qué son recomendados** y **qué umbrales mínimos aplican**. Una tarea individual puede exigir más, **nunca menos**.

Leyenda: **O** = obligatorio, **R** = recomendado, **N/A** = no aplicable, **C** = condicional según tamaño/parámetros de la tarea.

| Nivel | Exhaustiva | Aleatoria con semilla (≥ N) | Casos límite catalogados | Regresión (por circuito + familia) | Modelo de referencia | Pruebas por propiedades | Verificación modular | Validación incremental | Validación por señal semántica | Temporal |
|------:|:----------:|:---------------------------:|:------------------------:|:----------------------------------:|:--------------------:|:-----------------------:|:--------------------:|:----------------------:|:------------------------------:|:--------:|
| 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| 1 | O (≤2) | N/A | O (todo-cero, todo-uno) | O | N/A | N/A | N/A | R | N/A | N/A |
| 2 | O (≤4) | N/A | O | O | R | N/A | N/A | R | N/A | N/A |
| 3 | O (≤8) | C (si ≥3 entradas) | O | O | R | R | N/A | R | N/A | N/A |
| 4 | O (≤16) | R | O | O | R | R | R | O | R (por salida) | N/A |
| 5 | O (≤16) | R | O | O | O | R | R | O | R | N/A |
| 6 | C | O (≥ 1 000) | O por bus | O | O | R | R | O | O (etiquetas de bus) | N/A |
| 7 | O (≤256) | O (≥ 5 000) | O aritméticos | O | O | O | O por bloque | O | O (carry) | N/A |
| 8 | C (≤2¹⁶ por width) | O (≥ 10 000) | O por width | O por width | O | O | O | O | O (carry/sum) | N/A |
| 9 | C | O (≥ 10 000) | O por flag | O | O | O | O | O | O (todas las flags) | N/A |
| 10 | C (por opcode × valores) | O (≥ 20 000) | O por opcode | O | O | O | O por bloque | O | O (flags + opcode) | N/A |
| 11 | N/A para el todo | O (secuencias ≥ 1 000) | O reset/enable | O | O | O temporal | O por bloque | O | O temporal | O |
| 12 | N/A | O (≥ 5 000 secuencias) | O direcciones extremas | O | O | O | O | O | O (read/write) | O |
| 13 | N/A | O (≥ 10 000 ciclos en programas) | O programas mínimos | O | O (modelo arquitectónico) | O | O por bloque arquitectónico | O | O (PC, registros, flags) | O |

### Notas sobre la matriz

1. **Exhaustiva** se entiende como cubrir todas las combinaciones de entradas; en niveles donde el espacio es manejable (≤ ~2²⁰ con margen) sigue siendo factible. Por encima, se complementa con aleatoria + modelo + propiedades.
2. **Aleatoria con semilla** especifica un número mínimo de vectores aleatorios reproducibles; el `seed_strategy` de la tarea fija la semilla.
3. **Casos límite catalogados** crece con el sistema. Toda entrada que en algún momento detectó un bug en cualquier circuito de la familia se incorpora y permanece.
4. **Regresión** combina la suite específica del circuito y la suite de la familia.
5. **Modelo de referencia** vuelve obligatorio en cuanto entran sumadores/comparadores y se mantiene en niveles superiores.
6. **Pruebas por propiedades** son cuantificaciones (∀ vectores: prop. P se cumple). En nivel ≥ 4 son recomendables; ≥ 7 obligatorias.
7. **Verificación modular**: para circuitos con jerarquía conceptual (mux, full adders compuestos, ALUs, etc.) se verifica cada bloque internamente, luego la composición.
8. **Validación incremental**: durante construcción interactiva, re-verifica solo conos lógicos afectados por el último cambio. Recomendable desde nivel 1, obligatoria desde nivel 4.
9. **Validación por señal semántica**: si la tarea declara etiquetas como `carry`, `zero`, `overflow`, esas señales deben comportarse según su etiqueta. Obligatoria a partir del nivel donde aparecen.
10. **Temporal**: a partir del nivel 11 entra el modo temporal del simulador y la verificación se extiende a secuencias.

## Umbrales por defecto de cobertura aleatoria

| Bits de entrada combinacional | Exhaustiva viable | Vectores aleatorios mínimos si no exhaustiva |
|------------------------------:|:-----------------:|:--------------------------------------------:|
| ≤ 12 | Sí | — |
| 13–16 | Sí (con coste) | — |
| 17–20 | Límite | 5 000 |
| 21–24 | No | 10 000 |
| 25–32 | No | 20 000 |
| > 32 | No | 50 000 + dirigidas |

Estos umbrales son configurables por instalación; los **conceptos** son fijos.

## Composición de la decisión final del verificador

Para una tarea dada, el verificador entrega **PASA** si y solo si:

```
PASA  ⟺  ∀ suite ∈ task.required_test_suites :
            suite_resultado(circuito, suite) == PASA
       ∧  ∀ caso ∈ catalogo_de_casos_limite[task.level] :
            caso.blocking == true ⟹ caso_resultado(circuito, caso) == PASA
       ∧  ∀ propiedad ∈ task.properties :
            propiedad_resultado(circuito, propiedad) == PASA
       ∧  ∀ sig ∈ task.semantic_signals_to_check :
            sig.checker(circuito) == PASA
       ∧  comparacion_referencia(circuito, task.reference_model) == PASA   (si aplica)
       ∧  ¬regresion_contra_oficial_activo(circuito, task)
       ∧  (si tarea es temporal) verificacion_temporal(circuito, task) == PASA
```

Cualquier conjunción que falle ⇒ FALLA. Sin grises.

## Estrategias de selección de casos aleatorios

Cuando la aleatoria reemplaza a la exhaustiva, AONIX usa selección **estratificada**:

- Cobertura uniforme del espacio de entrada.
- Sobre-muestreo de regiones cercanas a casos límite conocidos.
- Mezcla de patrones (alternados, valores extremos, single-bit, etc.).
- Inclusión obligatoria de casos del catálogo si la tarea lo declara.

La selección está determinada por la semilla; misma semilla ⇒ mismos vectores ⇒ mismos resultados.

## Política sobre tiempos de verificación

| Nivel | Tiempo de verificación orientativo |
|-------|------------------------------------|
| 0–3 | < 100 ms |
| 4–6 | < 1 s |
| 7–10 | < 30 s |
| 11–13 | < 5 min (incluye secuencias temporales y programas) |

Los tiempos son orientativos. Si una verificación tarda significativamente más de lo previsto, AONIX no aborta automáticamente; reporta el tiempo en el reporte de verificación para análisis.

## Lo que la matriz **no** permite

- Reducir el conjunto obligatorio.
- Saltarse la verificación por señal semántica cuando hay etiquetas declaradas.
- Aceptar una solución parcialmente correcta como "casi pasa".
- Sustituir verificación funcional por mejora estructural (correctitud manda).
