options(repos = "http://cran.rstudio.com/")

# Make sure tcltk
install.packages("tcltk")
install.packages("renv")

# Initialize the project and create a project-specific library
renv::init()

# Install the packages required by your script
renv::install(c("RSQLite", "cpp11", "glue", "lifecycle", "memoise", "pkgconfig", "rlang"))

# Save a snapshot of the current environment to renv.lock
renv::snapshot()
