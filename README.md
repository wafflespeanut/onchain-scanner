
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
			"Resource": "arn:aws:lambda:*:*:function:FetchOnchainBars"
		}
	]
}
```

### Custom policy for `onchain-scanner`

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": "lambda:InvokeFunction",
            "Resource": "arn:aws:lambda:*:*:*"
        }
    ]
}
```
