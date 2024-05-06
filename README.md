# TheStack

A Rust stack, exploring Front-end, Back-end, Infra, Dev-Ops, Cloud all in one place.

## Required Tooling

* Cargo
* Docker

## Features

### The Stack Service

* REST API
  * Coupon
    * High level of concurrency
  * Metrics
    * Prometheus metrics
  * User Login
    * Password hashing
  * Worker
    * Changes an Arc<Mutext<>> via API that modifies worker behaviour
* Cache
  * Cache-Aside strategy for Coupons
* Database
  * PostgreSQL
    * Common Table Expression (CTE)
    * Procedures
    * UUID as PK
    * Batch insert using `unnest`
* Jobs
  * Cleanup worker job
    * Used coupons in cache are cleared from the database

### The Stack Tester

* Fetches coupons randomly from a set of coupon sets
* Tests the resilience of concurrent operations

## Scripts

* Use makefile as entry point
* All scripts are meant to be run at the root of the project
* Generate RSA private and public keys for JWT
