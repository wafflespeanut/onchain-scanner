

### Custom policy for `cargo-lambda` deployment

```json
{
	"Version": "2012-10-17",
	"Statement": [
		{
			"Effect": "Allow",
			"Action": [
				"iam:CreateRole",
				"iam:AttachRolePolicy",
				"iam:UpdateAssumeRolePolicy",
				"iam:PassRole"
			],
			"Resource": [
				"arn:aws:iam::${AWS_ACCOUNT}:role/AWSLambdaBasicExecutionRole",
				"arn:aws:iam::${AWS_ACCOUNT}:role/LambdaBasic"
			]
		},
		{
			"Effect": "Allow",
			"Action": [
				"lambda:CreateFunction",
				"lambda:UpdateFunctionCode",
				"lambda:GetFunction"
			],
			"Resource": "arn:aws:lambda:ap-east-1:${AWS_ACCOUNT}:function:FetchOnchainBars"
		}
	]
}
```

> Given that Lambda pricing is for running time and every millisecond counts, region is chosen based on HTTP response times obtained from https://check-host.net/
