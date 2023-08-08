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

variable "lambda_short_name" {
  type    = string
  default = "rigitbot"
}

locals {
  root_dir         = "${path.module}/.."
  lambda_full_name = "${var.lambda_short_name}-${data.aws_region.current.name}"
  lambda_bootstrap = "${local.root_dir}/target/lambda/rigitbot/bootstrap"
  lambda_zip       = "${local.lambda_bootstrap}.zip"
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
          "logs:PutLogEvents",
        ]
        Effect   = "Allow"
        Resource = "*"
      },
    ]
  })
}

resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${local.lambda_full_name}"
  retention_in_days = 180
}

resource "null_resource" "build_lambda" {
  # Thank-you to: https://medium.com/@jakub.jantosik/aws-lambda-and-rust-building-aws-lambda-functions-with-terraform-pt-1-a09e5c0a0cb9
  triggers = {
    cargo_file      = filesha256("${local.root_dir}/Cargo.toml")
    cargo_lock_file = filesha256("${local.root_dir}/Cargo.lock")
    src_dir         = sha256(join("", [for f in sort(fileset("${local.root_dir}/src", "**")) : filesha256("${local.root_dir}/src/${f}")]))
  }

  provisioner "local-exec" {
    command     = "cargo lambda build --release --arm64"
    working_dir = "${path.module}/.."
  }
}

data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = local.lambda_bootstrap
  output_path = local.lambda_zip
  depends_on  = [null_resource.build_lambda]
}

resource "aws_lambda_function" "lambda" {
  architectures = ["arm64"]
  publish       = true
  depends_on    = [aws_cloudwatch_log_group.lambda_log_group]
  filename      = local.lambda_zip
  function_name = local.lambda_full_name
  role          = aws_iam_role.lambda_execution_role.arn
  handler       = "bootstrap"

  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

  runtime = "provided.al2"
}

resource "aws_lambda_function_url" "lambda_url" {
  function_name      = aws_lambda_function.lambda.function_name
  authorization_type = "NONE"
}
