terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.11"
    }
  }
  required_version = "~> 1.5"
}

provider "aws" {
  region = "us-east-2"
}

data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

variable "lambda_short_name" {
  type    = string
  default = "rigitbot"
}

locals {
  region           = data.aws_region.current.name
  aws_account_id   = data.aws_caller_identity.current.account_id
  root_dir         = "${path.module}/.."
  lambda_full_name = "${var.lambda_short_name}-${data.aws_region.current.name}"
  lambda_zip = "${local.root_dir}/target/lambda/rigitbot/bootstrap.zip"
}


resource "aws_iam_role" "lambda_execution_role" {
  name = "${local.lambda_full_name}-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Sid    = ""
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      },
    ]
  })

  managed_policy_arns = [aws_iam_policy.lambda_execution_role_policy.arn]
}

resource "aws_iam_policy" "lambda_execution_role_policy" {
  name        = "${local.lambda_full_name}-execution-role-policy"
  path        = "/"
  description = "Policy for the execution role for ${local.lambda_full_name}"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents",
        ]
        Effect = "Allow"
        Resource = [
          "arn:aws:logs:${local.region}:${local.aws_account_id}:log-group:${aws_cloudwatch_log_group.lambda_log_group.name}",
          "arn:aws:logs:${local.region}:${local.aws_account_id}:log-group:${aws_cloudwatch_log_group.lambda_log_group.name}:*",
        ],
      },
    ]
  })
}

resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${local.lambda_full_name}"
  retention_in_days = 180
}

resource "aws_lambda_function" "lambda" {
  architectures = ["arm64"]
  publish       = true
  depends_on    = [aws_cloudwatch_log_group.lambda_log_group]
  filename      = local.lambda_zip
  function_name = local.lambda_full_name
  role          = aws_iam_role.lambda_execution_role.arn
  handler       = "bootstrap"

  source_code_hash = filebase64sha256(local.lambda_zip)

  runtime = "provided.al2"
}

resource "aws_lambda_function_url" "lambda_url" {
  function_name      = aws_lambda_function.lambda.function_name
  authorization_type = "NONE"
}
