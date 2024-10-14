if [[ -n $1 ]]; then
    WALLET_PATH=$1
else
    echo "Error: deployer not found: $WALLET_PATH"
    exit 1
fi

echo "Deploying program"

PROGRAM_NAME_UNDERSCORE=${SOL_PROGRAM_NAME//-/_}
solana program deploy target/deploy/$PROGRAM_NAME_UNDERSCORE.so --keypair $WALLET_PATH --url $CLUSTER_URL
