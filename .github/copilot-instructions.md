---
description: AI rules derived by SpecStory from the project AI interaction history
globs: *
---

## HEADERS

This document defines the rules and guidelines for the AI coding assistant within this project. It covers coding standards, project structure, workflow, and best practices. This file is a "living" document, evolving with the project. All contributions and modifications must adhere to the principles outlined here.

## PROJECT DESCRIPTION & GOALS

The project is a multi-tenant CMS/server template. The goal is a platform where content and site configuration are served per-tenant, with optional global defaults. This includes pages, products, works, inquiries, categories, media, and globals (Header/Footer/Theme/Commissions). The system will incorporate tenant-aware globals and role-based access control.

## TECH STACK

*   Rust (Backend)
*   Frontend (Unspecified - to be clarified if necessary)

## CODING STANDARDS

*   Adhere to Rust's idiomatic coding practices.
*   Code should be well-documented and easy to understand.
*   All code must be testable.
*   No introduction of new DB frameworks unless the repo already uses one. Tenant-scoped globals should follow the project's existing storage approach (likely in-memory, file-backed, or integrated with the same persistence used for Pages/Products, etc.).
*   Avoid introducing external example code or new frameworks. When implementing tenant-scoped globals, follow the project's existing storage approach (likely in-memory, file-backed, or integrated with the same persistence used for Pages/Products, etc.).

## PROJECT STRUCTURE

*   The backend is written in Rust.
*   The project uses a multi-tenant architecture.
*   Models, handlers, and potentially a `globals` module will be used for tenant-aware globals.

## WORKFLOW & RELEASE RULES

*   Use `pnpm lint` to check for linting errors.
*   Address linting warnings as part of the development process.

## DEBUGGING

*   Pay close attention to compiler warnings and errors.
*   Use debugging tools to identify and fix issues.
*   When fixing compile/lint problems, apply minimal, in-repo changes.

## TESTING

*   Write unit tests for all modules and functions.
*   Write integration tests to ensure that different parts of the system work together correctly.

## PROJECT DOCUMENTATION & CONTEXT SYSTEM

*   SpecStory is used to maintain a history of changes and decisions.
*   The AI assistant should read and summarize SpecStory history files to gather context and ensure consistency.
*   Filenames in history are wrapped in backticks.
*   Changes and fixes are captured in the history.

## AI ASSISTANT GUIDELINES

*   The AI assistant must adhere to the rules and guidelines outlined in this document.
*   The AI assistant should prioritize staying within the scope of the project.
*   The AI assistant should not generate example code that is not relevant to the project.
*   The AI assistant should ask for clarification when necessary.
*   The AI assistant should read the history to maintain context and avoid repeating previous mistakes.
*   The AI assistant should summarize key points and recommendations concisely, using bullet points with bold keywords and file paths wrapped in backticks.
*   When fixing compile/lint problems, the AI assistant should apply minimal, in-repo changes.
*   When suggesting fixes or changes, the AI assistant should provide clear instructions and ask for permission before proceeding.
*   The AI assistant must ask for permission before creating new models or tables.