# Shadow of War Server Rebuild

A reimplementation of the Shadow of War server infrastructure, providing a complete server solution for game state management and debugging capabilities.

## Overview

This project consists of three main components that work together to provide a robust server infrastructure:

- **Nemesis Server**: A Rust-based backend server handling game state and player interactions
- **Debugger**: A C++ debugging tool for analyzing network traffic and server behavior, as well as providing a way to easily dump requests and responses to files for later analysis
- **Request Tester**: A Python-based testing framework for validating server endpoints

## Project Structure

### Nemesis Server

A modern, high-performance server written in Rust using the Actix web framework.

#### Features
- RESTful API endpoints for game state management
- Configurable logging with JSON/plaintext formats
- Middleware for request tracking
- Compressed responses
- Custom binary protocol handling

#### Technical Details
- Configurable through environment variables
- Structured logging with rotation support
- Multi-threaded architecture with configurable worker count

### Debugger

A sophisticated C++ debugging tool for analyzing server behavior and network traffic.

#### Features
- Advanced logging system with request tracking
- Hydra value system for efficient data handling
- Windows API hooking for request/response interception
- Real-time traffic monitoring
- Request/Response logging to files

### Request Tester

A Python-based testing framework for validating server endpoints.

#### Features
- Custom AG Binary protocol implementation
- Automated endpoint testing
- Configurable request headers
- Detailed response analysis

#### Technical Details
- Supports custom binary protocols
- Configurable client parameters
- JSON response parsing

## Getting Started

### Prerequisites

- For Nemesis Server:
  - Rust toolchain
  - Cargo package manager
- For Debugger:
  - Modern C++ compiler
  - MinHook library
  - Windows SDK
- For Request Tester:
  - Python 3.x
  - Required Python packages (see requirements.txt)

### Installation

TODO

## Building

TODO

### Building the Nemesis Server

TODO

## Motivation
Upon hearing about the recent closure of Monolith Studios, I figure it would be both a fun and educational project to both reverse engineer the Shadow of War web protocol and then rebuild
the server infrastructure from scratch.

Shadow of War is one of my favorite games, and I would hate to see the backend server just die off. I am currently unsure how much time I can dedicate to this project, but I will do
my best to keep it updated and add new endpoints as I can.

## Contributing

I am open to contributions! Please feel free to submit a PR if you have any ideas or suggestions.