options(repos = "http://cran.rstudio.com/")

# Make sure tcltk
install.packages("tcltk")
install.packages("renv")
install.packages("stringi")
install.packages("caret")

# Initialize the project and create a project-specific library
renv::init()

# Install the packages required by your script
renv::install(c(
  "RSQLite", 
  "TTR", 
  "quantmod", 
  "xgboost", 
  "ROCR", 
  "dplyr", 
  "magrittr", 
  "here",
  "xts",
  "svglite",
  "neuralnet",
  "h2o"
))

# Save a snapshot of the current environment to renv.lock
renv::snapshot()
