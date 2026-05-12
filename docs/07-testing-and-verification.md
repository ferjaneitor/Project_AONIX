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
