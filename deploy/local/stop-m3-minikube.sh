#!/bin/sh
set -eu

profile=${PPL_MINIKUBE_PROFILE:-public-purpose-lab}
minikube stop --profile "$profile"
printf '%s\n' 'The Minikube VM is stopped; its cluster and M3.4 state are retained.'
