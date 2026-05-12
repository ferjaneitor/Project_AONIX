# 00 — Visión y principio rector

## Definición

**AONIX (AND-OR-NOT Integrated eXploration)** es una plataforma formal, determinista, 2D, escalable y verificable. Su propósito es ser el **mundo formal** donde agentes humanos, buscadores automáticos y modelos de IA aprendan a construir, simular, verificar, optimizar, visualizar y explicar circuitos digitales utilizando exclusivamente tres compuertas primitivas: **AND, OR y NOT**.

AONIX no es un simulador genérico, ni una biblioteca de compuertas, ni una herramienta CAD, ni un modelo de IA. AONIX es un **entorno formal de aprendizaje** sobre el cual una IA puede operar, fallar, aprender y mejorar bajo reglas absolutas e inviolables.

## Principio rector absoluto

> La IA propone, pero AONIX determina la verdad técnica.

Este principio no admite excepciones, atajos ni reinterpretaciones.

Ningún modelo neuronal, agente, heurística aprendida ni proceso estocástico puede declarar por sí mismo que un circuito es correcto, válido, óptimo, aceptable, equivalente, mejorado o canónico. **Esa autoridad pertenece exclusivamente a los módulos deterministas de AONIX**:

- Validador de acciones
- Simulador
- Verificador
- Evaluador estructural
- Sistema de pruebas
- Sistema de memoria
- Coordinador central

Si una IA propone un circuito, AONIX lo simula, lo verifica, lo mide y lo compara. Si pasa, gana. Si no pasa, no entra. La narrativa que la IA construya alrededor de su propuesta es irrelevante; solo cuentan los hechos formales producidos por los módulos deterministas.

## Reparto de roles

| Componente | Rol |
|-----------|-----|
| **AONIX** | Entorno formal del mundo |
| **IA** | Agente / jugador |
| **Circuito** | Construcción |
| **Tarea** | Misión |
| **Tabla de verdad / spec** | Meta |
| **Simulador** | Ejecuta |
| **Verificador** | Decide |
| **Evaluador** | Mide |
| **Memoria** | Guarda |
| **Currículo** | Define niveles |
| **Visualización** | Muestra |
| **Plataforma** | Traduce entre humanos, máquinas e IA |

AONIX debe **funcionar plenamente sin IA**. La IA vendrá después y usará AONIX como entorno de aprendizaje. Esta independencia operativa es deliberada: garantiza que el mundo formal pueda evolucionar, ser auditado y validado sin depender de pesos neuronales, y permite que diferentes agentes (humanos, búsqueda exhaustiva, SAT solvers, IA generativa, IA de refuerzo) usen el mismo entorno con las mismas reglas.

## Objetivos de la plataforma

AONIX debe permitir:

- Representar circuitos digitales como grafos 2D de AND/OR/NOT.
- Crear tareas con metas formales (tablas de verdad, propiedades, especificaciones).
- Definir metas y niveles tipo videojuego.
- Simular circuitos sobre entradas específicas o lotes.
- Validar acciones antes de ejecutarlas.
- Verificar comportamiento contra especificación.
- Medir calidad estructural (conteo, profundidad, reutilización, señales muertas).
- Optimizar agresivamente preservando comportamiento.
- Guardar memoria técnica con múltiples roles separados.
- Mantener historial de evolución con versiones históricas verificadas.
- Generar visualizaciones 2D mediante Vulkan.
- Traducir circuitos a lenguaje humano.
- Traducir estados a lenguaje consumible por IA.
- Organizar niveles tipo videojuego.
- Permitir entrenamiento futuro de IA.
- Escalar desde lógica booleana simple hasta arquitecturas de procesador.

## Lo que AONIX **no** es

- **No es una biblioteca de compuertas derivadas.** XOR, XNOR, NAND y NOR no existen como primitivas en AONIX.
- **No es una IA.** AONIX es el mundo formal. La IA es un agente externo opcional.
- **No es una herramienta de captura esquemática tradicional.** Es un entorno de aprendizaje gobernado por reglas absolutas.
- **No es un atajo.** AONIX no entrega soluciones; entrega un mundo donde aprenderlas, fallarlas, validarlas y mejorarlas.
- **No es un repositorio mutable de primitivas.** El conjunto de primitivas es fijo y no negociable: AND, OR, NOT.

## Filosofía de diseño

1. **Determinismo total.** Misma entrada + mismo circuito ⇒ mismo resultado, siempre. Sin ambigüedad temporal, sin orden de evaluación opaco, sin estado oculto.
2. **Verdad técnica externa a la IA.** La IA puede proponer; los módulos deterministas deciden. La separación es estricta.
3. **Composición sin atajo.** Cualquier circuito complejo (multiplexor, full adder, ALU, CPU) es válido si y solo si está expandido internamente a AND/OR/NOT.
4. **Semántica abierta vía etiquetas.** En lugar de prohibir conceptos arbitrarios (buses, flags, clock), AONIX usa etiquetas semánticas que informan al simulador, verificador, visualizador y traductor sin añadir primitivas.
5. **Memoria sin atajo.** AONIX guarda versiones históricas, trayectorias y aprendizajes, pero la construcción final permanece expresada solo con AND/OR/NOT. La memoria no se convierte en biblioteca de compuertas.
6. **Pruebas proporcionales al nivel.** Cuanto más alto el nivel, más capas de prueba debe superar el circuito.
7. **Optimización agresiva pero conservadora del comportamiento.** Una optimización solo reemplaza la versión oficial activa si pasa el sistema de pruebas correspondiente.
8. **Trazabilidad completa.** Cada decisión, prueba, optimización y reemplazo queda registrado y auditable.

## Resultado esperado

Una plataforma sobre la cual una IA pueda **aprender lógica booleana real**, construir estructuras cada vez más complejas, descubrir comportamientos derivados (incluyendo el XOR como composición de AND/OR/NOT, sin convertirlo en primitiva) y eventualmente sintetizar arquitecturas digitales completas — siempre desde tres únicos átomos lógicos.

AONIX es el laboratorio. Los circuitos son los experimentos. AND, OR y NOT son las partículas elementales.
