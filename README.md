# z3 app

## One binary, two static files : A full stack web app

Using rust, htmx, axum, askama, diesel and axum-login, you get the power of a full stack web app without the bloat.

## Features

- **Rust**: The backend is built with Rust, ensuring performance and safety.
- **HTMX**: Enables dynamic content loading and interaction without the need for a full JavaScript framework, making the frontend responsive and interactive.
- **Axum**: A web framework for building APIs and web applications in Rust, providing a simple and efficient way to handle HTTP requests.
- **Askama**: Templating engine for rendering HTML views.
- **Diesel**: ORM for database interactions, providing type safety and compile-time checks.
- **Axum-login**: Authentication middleware for secure user management.
- **Tailwind CSS**: Utility-first CSS framework for styling the frontend.
- **Docker**: Containerization for easy deployment and scalability.
- **PostgreSQL**: Relational database for data storage.

Everything is done server-side, so you don't need to worry about client-side JavaScript frameworks. The HTML is rendered on the server and sent to the client, ensuring a fast and responsive user experience.

As a result, you can focus on building your application logic without getting bogged down in frontend complexities.

Because it only uses rust and htmx, the project remains lightweight and easy to maintain. The bundle size is minimized, and there are fewer dependencies to manage.

Also, rust's strong type system and compile-time checks help catch errors early in the development process, leading to more robust and reliable code. On top of that, the application will be blazingly fast, thanks to Rust's performance caracteristics.

## Getting Started

To get started with this project, follow these steps:

### Prerequisites

Make sure you have the following installed:

- **Rust** (with Cargo)
- **Docker** (optional, for running the development database)
- **Diesel CLI**

#### 1. Clone the repository

```bash
git clone https://github.com/ZiedYousfi/z3-app.git
cd z3-app
```

#### 2. Set up the environment

Copy the example environment file:

```bash
cp .env.example .env
```

#### 3. Set up the DEVELOPMENT database

If you have Docker installed, you can start a PostgreSQL database with:

```bash
chmod +x ./start-database.sh
./start-database.sh
```

If you don't have Docker, set up a PostgreSQL database locally and update the connection settings in your `.env` file.

Apply the database migrations:

```bash
diesel migration run
```

#### 4. Build the project

   ```bash
   cargo build
   ```

#### 5. Run the application

   ```bash
   cargo run
   ```

#### 6. Access the application in your web browser at `http://localhost:3000`

### Docs

The tools and libraries used in this project were all chosen for their extensive documentation. Here are some useful links to get you started:

- [Rust](https://www.rust-lang.org/learn)
- [HTMX](https://htmx.org/docs/)
- [Axum](https://docs.rs/axum/latest/axum/)
- [Askama](https://docs.rs/askama/latest/askama/)
- [Diesel](https://diesel.rs/guides/getting-started/)
- [Axum-login](https://docs.rs/axum-login/latest/axum_login/)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [Docker](https://docs.docker.com/get-started/)
- [PostgreSQL](https://www.postgresql.org/docs/)

But the best way to start in my opinion is this video I made, which walks you through the entire process of using this template to build a todo app with this stack:

[![Watch the video](https://img.youtube.com/vi/your-video-id/maxresdefault.jpg)](https://www.youtube.com/watch?v=your-video-id)

The code is also very descriptive, with comments and documentation throughout. You can find the main application logic in the `src` directory, and the HTML templates in the `templates/html` directory if that's your style of discovering.

## It IS a template

This project is a template for building full stack web applications in Rust. It provides a solid foundation for creating web applications with user authentication, database interactions, and a responsive frontend. You can use it as a starting point for your own projects, customizing the features and functionality to suit your needs.

It's just supposed to be a starting point, so you can add your own features and functionality as needed. The project is designed to be modular and extensible, allowing you to easily integrate additional libraries and frameworks as required. You could even add react or vue.js or svelte if you really want to, but I don't see the point of doing that.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE.md) file for details.

## Contributing

If you'd like to contribute to this project, feel free to submit a pull request or open an issue. Contributions are welcome and appreciated!
