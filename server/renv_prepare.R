options(repos = "http://cran.rstudio.com/")

# Make sure tcltk
install.packages("tcltk")
install.packages("renv")
install.packages("stringi")
install.packages("keras")

# install_keras function has several arguments as follows:

  # install_keras(method = c("auto", "virtualenv", "conda"),
  #  conda = "auto", tensorflow = "default",
  #   extra_packages = c("tensorflow-hub"))
library(keras)
use_virtualenv("./keras_tf_env", required = TRUE)

# if you wish to enjoy your GPU, you are welcomed to change the configuration and specify tensorflow = “gpu”.

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
  "neuralnet"
))

# Save a snapshot of the current environment to renv.lock
renv::snapshot()
