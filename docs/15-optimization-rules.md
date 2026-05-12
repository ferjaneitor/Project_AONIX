# 15 — Reglas de optimización

> **Documento normativo.** Define qué es optimización en AONIX, qué transformaciones son legítimas, qué garantías deben preservarse, qué se mide, cómo se decide reemplazar el oficial activo, y qué hacer cuando una optimización falla.

## Principio

> Optimizar es mejorar sin mentir.

En AONIX, **optimización** significa transformar un circuito para mejorar sus métricas estructurales **preservando exactamente** su comportamiento sobre la spec de la tarea. Cualquier cambio que altere el comportamiento dentro del dominio relevante **no es optimización**, es **defecto introducido**.

Optimización es deseada, agresiva, sistemática. Pero la correctitud es invariante: la jerarquía absoluta de la recompensa (correctitud > optimización > elegancia > velocidad) se aplica también aquí. Una optimización que rompe el verificador queda **descartada**, no "discutida".

## Qué se optimiza

Las métricas legítimas a mejorar:

1. **Número de compuertas** (total y por tipo). Menor es mejor.
2. **Profundidad lógica** (longitud del camino crítico). Menor es mejor.
3. **Señales muertas** (no alcanzables desde ninguna salida). Cero es la meta en oficial activo.
4. **Reutilización de subexpresiones** entre salidas. Más es mejor.
5. **Compartición de señales internas.** Más es mejor.
6. **Fan-out máximo** controlado donde aplique (límites razonables; reducir picos extremos).
7. **Costo agregado** ponderado (función configurable).
8. **Complejidad visual estimada** (proxy de legibilidad humana). Menor es mejor, sin sacrificar lo anterior.
9. **Costo de simulación** y **costo de validación**. Menor es mejor.

## Qué **no** es optimización

- **Renombrar señales** sin cambio estructural. Es maquillaje; no se contabiliza como optimización.
- **Reordenar el grafo** sin reducir nodos ni profundidad. No es optimización.
- **Sustituir un nodo por una compuerta derivada.** Esto **no existe** como opción: las primitivas son cerradas.
- **Eliminar nodos sin re-verificar.** Eliminar lo que no es muerto es romper el circuito.
- **Optimizar para una métrica sacrificando correctitud.** Eso es regresión, no optimización.
- **Optimizar parcialmente algunas salidas.** O preserva el comportamiento entero o no es optimización válida.

## Catálogo de transformaciones legítimas

Cada transformación tiene **precondición**, **efecto** y **garantía de preservación de comportamiento**. Lista no cerrada: se amplía con la experiencia, siempre cumpliendo la garantía.

### Eliminaciones

- **Eliminación de señales muertas.** Precondición: una señal no alcanza ninguna salida del circuito. Efecto: eliminar la señal y sus compuertas productoras si no alimentan otra salida viva. Garantía: comportamiento idéntico en toda entrada.
- **Eliminación de compuertas redundantes.** Precondición: una compuerta produce un valor constante o duplica otra señal disponible. Efecto: sustituir referencias por la fuente equivalente. Garantía: idéntica.
- **Eliminación de doble negación.** Precondición: `NOT(NOT(x))`. Efecto: reemplazar por `x`. Garantía: idéntica.

### Aplicación de leyes booleanas (preservan comportamiento por definición)

- **Idempotencia.** `A AND A → A`, `A OR A → A`.
- **Identidad.** `A AND 1 → A`, `A OR 0 → A`.
- **Aniquilación.** `A AND 0 → 0`, `A OR 1 → 1`.
- **Absorción.** `A AND (A OR B) → A`, `A OR (A AND B) → A`.
- **De Morgan.** `NOT(A AND B) → NOT A OR NOT B`, `NOT(A OR B) → NOT A AND NOT B` (cuando reduce profundidad o conteo, no por capricho).
- **Distributividad** controlada, cuando reduce métricas.
- **Complemento.** `A AND NOT A → 0`, `A OR NOT A → 1`.

### Factorización y reutilización

- **Detección de subexpresiones comunes (CSE).** Precondición: dos o más subgrafos producen la misma señal con la misma fórmula sobre las mismas entradas. Efecto: sustituirlos por una sola compuerta compartida con fan-out aumentado.
- **Compartición entre salidas.** Si dos salidas comparten un cono interno, fundir el cono.
- **Factorización booleana.** `(A AND B) OR (A AND C) → A AND (B OR C)` cuando reduce conteo o profundidad.

### Reducciones de profundidad

- **Re-asociación de AND/OR n-arios** equivalentes para reducir profundidad lógica (transformar cadenas largas en árboles balanceados).
- **Hoisting** de operaciones comunes a niveles superiores cuando reduce camino crítico.

### Reducciones específicas de NOT

- **Empuje de NOT** hacia la frontera (entradas) o hacia el interior según convenga al tamaño y profundidad.
- **Fusión de NOT** consecutivos.

## Transformaciones **prohibidas**

Las siguientes son tentaciones que AONIX rechaza por construcción:

1. **Sustituir bloque equivalente a XOR por un nodo `XOR`.** No existe ese nodo. La sustitución crearía una compuerta prohibida.
2. **Sustituir bloque equivalente a NAND por un nodo `NAND`.** Idem.
3. **Sustituir bloque por una "compuerta derivada de la biblioteca".** No hay tal biblioteca de primitivas.
4. **Sustituir subcircuito por una invocación opaca de circuito compuesto guardado.** La versión oficial activa permanece expandida a primitivas. La invocación jerárquica puede existir en herramientas de organización, pero la **representación canónica** del oficial activo siempre se serializa expandida.
5. **Eliminar verificación post-optimización.** Toda optimización dispara re-verificación obligatoria.
6. **Marcar como optimización un cambio cuya equivalencia no esté probada.** Las equivalencias provienen de leyes booleanas o de prueba exhaustiva/aleatoria en el dominio relevante.

## Garantía de preservación de comportamiento

Toda transformación legítima debe garantizar:

```
∀ vector de entrada v ∈ Domain(tarea) :
    simular(circuito_original, v) == simular(circuito_transformado, v)
```

La garantía se asegura por **dos vías complementarias**, no por una sola:

1. **Vía algebraica.** La transformación corresponde a una ley booleana del catálogo de transformaciones legítimas. La equivalencia se prueba a priori por construcción.
2. **Vía verificación.** Tras la transformación, el verificador reejecuta **todas las suites de la tarea**. Si alguna falla ⇒ regresión ⇒ descarte.

**Las dos vías son obligatorias.** La vía algebraica garantiza que la transformación es legítima en abstracto; la vía verificación garantiza que la aplicación concreta no introdujo bug.

## Pipeline de optimización

Cuando un circuito candidato pasa verificación inicial, el coordinador invoca el optimizador estructural:

```
1.  CANDIDATO original (ya verificado)
        │
        ▼
2.  OPTIMIZADOR aplica transformaciones del catálogo:
    - en orden de costo (eliminaciones primero, factorizaciones después,
      re-asociaciones al final)
    - cada transformación es atómica
    - cada transformación tiene tipo de cambio registrado
        │
        ▼
3.  Tras cada transformación, hash del grafo cambia ⇒ marca dirty
        │
        ▼
4.  Iteración hasta punto fijo (ninguna transformación aplicable
    produce mejora medible) o tope de iteraciones
        │
        ▼
5.  RE-VERIFICACIÓN COMPLETA del circuito optimizado contra todas las
    suites de la tarea
        │
        ▼
6.  Si pasa: candidato optimizado
    Si falla en alguna suite que el candidato pre-optimización pasaba:
        REGRESIÓN ⇒ retroceder
        ⇒ marcar la transformación culpable en memoria de optimización
        ⇒ retomar desde el último estado verificado
```

## Memoria de optimización

Toda transformación aplicada se registra en la **memoria de optimización** (ver [05](05-memory-system.md) y [18](18-operational-vs-non-operational-memory.md)) con:

- Hash antes y hash después.
- Tipo de transformación aplicada.
- Métricas antes y después (delta).
- Resultado de re-verificación.
- Si fue descartada: causa.

Esta memoria es **append-only**. Sirve para:

- Aprender qué transformaciones son productivas en qué contextos.
- Detectar regresiones del propio optimizador.
- Entrenar modelos que aprendan a sugerir transformaciones.
- Auditar la evolución estructural de circuitos canónicos.

## Búsqueda dirigida por la estructura

El optimizador no es un solver genérico. Es una **estrategia con backtracking**:

1. Catálogo finito de transformaciones.
2. Detección de oportunidades por patrones en el grafo.
3. Aplicación greedy con re-verificación.
4. Backtracking si una rama empeora.
5. Tope de iteraciones por tarea (configurable).

Para tareas grandes, el optimizador puede:

- **Particionar por cono lógico** y optimizar conos por separado.
- **Compartir trabajo** detectando subgrafos comunes entre conos.
- **Aplicar reducciones locales** antes que globales (más baratas, suelen converger).

## Optimizador no determinista controlado

El optimizador es **determinista** sobre la misma versión del catálogo de transformaciones y el mismo input. No usa azar.

Si en el futuro se introducen heurísticas dependientes de azar (ej. orden aleatorio de aplicación), deberán:

- Usar semilla explícita.
- Registrar la semilla en el reporte.
- Garantizar misma semilla ⇒ mismo resultado.

## Reemplazo de oficial activo

El circuito optimizado solo reemplaza al oficial activo si:

1. Pasa la puerta de aceptación funcional (ver [13](13-circuit-acceptance.md)).
2. **Mejora estricta** según `ranking_order` de la tarea (ver [13](13-circuit-acceptance.md) §28).
3. No introduce regresión (ver [13](13-circuit-acceptance.md) §27 y [14](14-circuit-rejection.md) §L1.8).
4. Pasa la puerta de promoción del coordinador (ver [14](14-circuit-rejection.md) §L4).
5. El reemplazo es **atómico transaccional**.

La versión anterior se archiva en memoria histórica (ver [19](19-versioning-policy.md)).

## Política de descarte de optimizaciones

Cuando una transformación falla la re-verificación:

- **Se descarta esa transformación**, no el resto del trabajo.
- **Se anota la causa** en memoria de optimización.
- **Si el patrón persiste**, se marca la transformación como **sospechosa** en el catálogo para revisión.
- **Si todas las transformaciones disponibles fallan**, el candidato original queda como propuesta (sin optimizar, pero verificado).

Nunca se "fuerza" una optimización que rompe el verificador.

## Optimización de tareas con `TemporalSpec`

En tareas temporales:

- Las transformaciones se aplican solo sobre la parte combinacional.
- Las señales `clock`, `reset`, `enable` y el feedback temporal se preservan estructuralmente.
- La verificación temporal completa se ejecuta tras la optimización.
- Cambios que toquen el bucle temporal requieren prueba adicional (estabilidad, ausencia de glitches modelados).

## Optimización y aprendizaje

La memoria de optimización es input legítimo del entrenamiento de IA:

- Pares (antes, después, transformación) son ejemplos supervisados.
- Trayectorias de optimización son ejemplos para aprender estrategia.
- Casos de descarte son ejemplos negativos.

Nada de esto cambia las reglas absolutas: la IA puede aprender a sugerir transformaciones, pero **toda transformación aplicada pasa por el verificador**. No hay confianza implícita en la IA.

## Lo que la optimización **nunca** debe producir

- Un circuito con compuerta distinta de AND/OR/NOT.
- Un circuito que falla una prueba que el original superaba.
- Un circuito que mejora métricas pero pierde semántica de señal etiquetada (ej. la salida `carry` deja de comportarse como acarreo).
- Un circuito cuyo hash canónico se calcule mal por una transformación parcial.
- Un circuito visualmente "agrupado" que sugiera primitivas falsas (la visualización no toca el grafo; ver [09](09-visualization-vulkan.md)).
