# Kush

Remote execution Swiss Army knife.

## Target Parsing Notes

Perform helpful speculation on what the given value may be. For example:
- Assume `arn:` prefixes are ARNs and parse them accordingly
- Assume `i-` prefixes are EC2
- Assume `-[a-z0-9]{10}-[a-z0-9]{5}` suffixes are K8s pods

[URI vs URL](https://danielmiessler.com/p/difference-between-uri-url/)
