# 04 — Formato `.aonclg`

## Identidad del formato

**`.aonclg`** — **AON C**anonical **L**earning **G**raph (también descrito como AON Circuit Learning Graph).

Es el documento de **aprendizaje para IA** asociado a un circuito. **No es** una segunda versión del circuito. Es el **contexto de aprendizaje** que rodea al `.aoncir`.

## Propósito

El `.aonclg` debe:

- Ayudar a una IA a aprender lógica booleana usando AONIX como entorno.
- Conservar el contexto formativo de cada circuito: meta, niveles, errores, trayectorias, recompensas.
- Permitir reproducir episodios de aprendizaje (semillas, secuencias de acciones).
- Servir de currículo personal del agente sobre un circuito específico.
- Alimentar entrenamiento futuro (aprendizaje por refuerzo, supervisado, autoaprendizaje).

El `.aonclg` **no puede**:

- Introducir compuertas nuevas.
- Sustituir al `.aoncir` como fuente de verdad técnica.
- Modificar el grafo del circuito.
- Servir como atajo que evite reconstruir lógica desde AND/OR/NOT.

## Relación con `.aoncir`

```
            ┌──────────────┐
            │  Tarea       │
            └──────┬───────┘
                   │
        ┌──────────┴───────────┐
        ▼                      ▼
┌───────────────┐      ┌───────────────────┐
│  .aoncir      │◄─────┤  .aonclg          │
│  (verdad)     │ ref  │  (aprendizaje)    │
└───────────────┘      └───────────────────┘
```

- Un `.aoncir` puede existir sin `.aonclg` (circuito sintetizado por búsqueda exhaustiva, por ejemplo).
- Un `.aonclg` siempre referencia al menos un `.aoncir` (final, intermedio o histórico).
- Un `.aoncir` puede tener múltiples `.aonclg` asociados (uno por episodio, por agente, por nivel, etc.).

## Estructura lógica

```
1. Metadatos del documento
   - id del .aonclg
   - referencia al .aoncir asociado (por hash canónico)
   - referencia a tarea de origen
   - nivel curricular
   - agente (humano | búsqueda | modelo de IA con versión)
   - semilla aleatoria
   - timestamp de inicio y cierre del episodio

2. Meta de la tarea
   - descripción formal
   - tabla de verdad o spec resumida
   - entradas y salidas con etiquetas semánticas
   - criterios de éxito

3. Estado inicial
   - circuito parcial inicial (puede estar vacío)
   - señales disponibles al inicio
   - acciones permitidas al inicio

4. Trayectoria del episodio
   - secuencia ordenada de:
       { action_proposed, action_legal?, validator_feedback,
         circuit_state_after, simulator_output_if_applicable,
         verifier_partial_status, evaluator_partial_metrics,
         reward_delta }

5. Recompensas
   - desglose por componente (correctitud, optimización, elegancia, velocidad)
   - reward total final
   - puntuación curricular

6. Errores comunes detectados
   - patrones inválidos intentados
   - causas plausibles (categorizadas, no especulativas)

7. Casos fallidos y casos límite
   - entradas que produjeron salida incorrecta durante el camino
   - casos límite probados
   - casos límite superados/no superados

8. Pruebas requeridas y resultado
   - suite_id por nivel
   - pasadas / fallidas
   - regresiones detectadas

9. Criterios de avance
   - condiciones de éxito del nivel
   - cuáles se cumplieron y cuáles no

10. Representación para IA
    - serialización del estado en cada paso adaptada al consumo del agente
    - lista de acciones legales por estado
    - features estructurales del circuito parcial

11. Información visual abstracta
    - hints de layout, agrupaciones, regiones
    - sin píxeles — solo referencias semánticas

12. Relación con tareas anteriores
    - tareas prerrequisito superadas
    - circuitos reutilizados como subcomponentes (referencia a .aoncir guardados)

13. Historial post-episodio
    - hash del .aoncir final si el episodio acabó en circuito válido
    - hash del .aoncir oficial activo al momento del cierre
    - si esta solución reemplazó al oficial activo: hash anterior y delta

14. Etiquetas de aprendizaje (clasificación libre, controlada por la plataforma)
    - dificultad percibida
    - tipo de estrategia detectada (factorización, reutilización, fuerza bruta)
    - calidad del aprendizaje extraído

15. Metadatos de reproducibilidad
    - versión de AONIX
    - versión del .aoncir asociado
    - versión del modelo de IA
    - parámetros del agente
```

## Reglas absolutas del `.aonclg`

1. **No introduce compuertas nuevas.** Todo lo que aparece en un `.aonclg` referencia el grafo del `.aoncir` asociado, no lo modifica.
2. **No sustituye al `.aoncir`.** Un agente que lee un `.aonclg` para aprender no puede saltarse la construcción real del circuito.
3. **No puede contener spoilers como atajo.** El `.aonclg` puede contener la solución (el `.aoncir` final) porque sirve para aprendizaje supervisado o post-mortem, pero las herramientas que sirven `.aonclg` a un agente en entrenamiento durante un episodio activo deben filtrar el spoiler salvo cuando el currículo lo permita explícitamente (p. ej. modo "estudiar antes de practicar").
4. **Es inmutable una vez cerrado el episodio.** Modificarlo crearía deriva de aprendizaje irrastreable.
5. **Es auditable.** Cada acción registrada incluye el resultado del validador, no solo lo propuesto por el agente.

## Modos de uso

El `.aonclg` cumple múltiples roles según el consumidor:

- **Para revisión humana** — explica por qué un agente tomó cada decisión y dónde se equivocó.
- **Para entrenamiento supervisado** — pares (estado, acción_correcta) o (estado, trayectoria_completa).
- **Para aprendizaje por refuerzo** — secuencia (estado, acción, recompensa, estado_siguiente).
- **Para análisis comparativo** — comparar trayectorias de distintos agentes sobre la misma tarea.
- **Para depuración de currículo** — detectar tareas donde todos los agentes fallan de la misma forma.
- **Para evolución del propio AONIX** — qué etiquetas semánticas se usan, qué pruebas se piden, qué tipos de errores aparecen.

## Decisión pendiente: representación física

Mismas alternativas que `.aoncir` (texto / binario / doble). Probablemente `.aonclg` se beneficie de un formato más permisivo en texto porque su contenido es más heterogéneo y se consulta más como log.

**Recomendación inicial:** texto estructurado (TOML o JSON) en fases tempranas; binario opcional si el volumen lo justifica.

## Lo que distingue `.aonclg` de un log genérico

Un log genérico es texto libre. Un `.aonclg`:

- Es **tipado** — toda entrada cumple un esquema.
- Es **enlazable** — referencia hashes canónicos de circuitos.
- Es **reproducible** — con la misma semilla y el mismo agente, regenera la trayectoria.
- Es **explotable** — herramientas de AONIX pueden consultarlo programáticamente.
- Es **separable del circuito** — borrar `.aonclg` no invalida el `.aoncir`; borrar `.aoncir` invalida los `.aonclg` que lo referencian (orphan).

## Política de retención

Los `.aonclg` no se borran automáticamente. La memoria de aprendizaje (ver [05 — Memorias](05-memory-system.md)) los conserva organizados por tarea, nivel y agente. Se pueden archivar o comprimir, pero no eliminar sin política explícita.

## Privacidad y trazabilidad

Si una IA externa genera un `.aonclg`, el archivo registra qué modelo (familia, versión, parámetros públicos) lo produjo. La autoría es siempre rastreable. No se asume confidencialidad sobre el contenido del aprendizaje: AONIX es un entorno de auditoría plena.
