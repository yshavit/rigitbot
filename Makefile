build:
	cargo lambda build --release --arm64 --output-format zip
	zip target/lambda/rigitbot/bootstrap.zip Rocket.toml

tf-plan: build
	terraform -chdir=infra plan

tf-apply: build
	terraform -chdir=infra apply -auto-approve
