#' Predict neural net
#'
#' @param nn_model 
#' @param data_to_predict 
#'
#' @return data.frame
#' @export
#'
#' @examples
#' predict_nnet(nnet_model, x_train)

predict_nnet <- function(nn_model, data_to_predict){ 
  
  #select repetition of neural network with minimal error 
  which_rep <- which(
    nn_model$result.matrix[1, ]== min(nn_model$result.matrix[1, ])) 
  
  # Compute output of the neural net
  prediction <- data.frame(
    stats::predict(
      object = nn_model,
      newdata = data_to_predict, 
      rep = which_rep)) 
  
  labels <- colnames(nn_model$response)
  # Find maximal probability and label it
  prediction_vector <- data.frame(max.col(prediction))
  prediction_vector$prediction <- labels[prediction_vector$max.col.prediction.]
  prediction_vector <- prediction_vector[, "prediction", drop = FALSE]
  
  prediction <- data.frame(
    signals = prediction_vector)
  return(prediction)
}
