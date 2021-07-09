import json
from http import HTTPStatus
from cryptography.fernet import Fernet
import base64
import base58
import struct

from solana.publickey import PublicKey 
from solana.transaction import Transaction, AccountMeta, TransactionInstruction
from solana.account import Account 
from solana.rpc.api import Client
import solana.rpc.types as types
from solana.system_program import transfer, TransferParams, create_account, CreateAccountParams 
from spl.token._layouts import MINT_LAYOUT, ACCOUNT_LAYOUT
from spl.token.instructions import (
    mint_to, MintToParams,
    initialize_mint, InitializeMintParams,
)

SYSTEM_PROGRAM_ID = '11111111111111111111111111111111'
SYSVAR_RENT_ID = 'SysvarRent111111111111111111111111111111111'
ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
TOKEN_PROGRAM_ID = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
BETTING_POOL_PROGRAM_ID = 'DnVoDXeLS9wmWWRk2LZZhWP4y7TxcVrwYhDaY7a6PS53'


def create_associated_token_account_instruction(associated_token_account, payer, wallet_address, token_mint_address):
    keys = [
        AccountMeta(pubkey=payer, is_signer=True, is_writable=True),
        AccountMeta(pubkey=associated_token_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=wallet_address, is_signer=False, is_writable=False),
        AccountMeta(pubkey=token_mint_address, is_signer=False, is_writable=False),
        AccountMeta(pubkey=PublicKey(SYSTEM_PROGRAM_ID), is_signer=False, is_writable=False),
        AccountMeta(pubkey=PublicKey(TOKEN_PROGRAM_ID), is_signer=False, is_writable=False),
        AccountMeta(pubkey=PublicKey(SYSVAR_RENT_ID), is_signer=False, is_writable=False),
    ]
    return TransactionInstruction(keys=keys, program_id=PublicKey(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID))

def initialize_betting_pool_instruction(
    pool_account,
    escrow_mint_account,
    escrow_account,
    long_token_mint_account,
    short_token_mint_account,
    mint_authority_account,
    update_authority_account,
    token_account,
    system_account,
    rent_account,
    decimals
):
    keys = [
        AccountMeta(pubkey=pool_account, is_signer=True, is_writable=True),
        AccountMeta(pubkey=escrow_mint_account, is_signer=False, is_writable=False),
        AccountMeta(pubkey=escrow_account, is_signer=True, is_writable=True),
        AccountMeta(pubkey=long_token_mint_account, is_signer=True, is_writable=False),
        AccountMeta(pubkey=short_token_mint_account, is_signer=True, is_writable=False),
        AccountMeta(pubkey=mint_authority_account, is_signer=True, is_writable=False),
        AccountMeta(pubkey=update_authority_account, is_signer=True, is_writable=False),
        AccountMeta(pubkey=token_account, is_signer=False, is_writable=False),
        AccountMeta(pubkey=system_account, is_signer=False, is_writable=False),
        AccountMeta(pubkey=rent_account, is_signer=False, is_writable=False),
    ]
    data = struct.pack("<BB", 0, decimals)
    return TransactionInstruction(keys=keys, program_id=PublicKey(BETTING_POOL_PROGRAM_ID), data=data)

def trade_instruction(
    pool_account,
    escrow_account,
    long_token_mint_account,
    short_token_mint_account,
    buyer,
    seller,
    buyer_account,
    seller_account,
    buyer_long_token_account,
    buyer_short_token_account,
    seller_long_token_account,
    seller_short_token_account,
    escrow_authority_account,
    token_account,
    size,
    buyer_price,
    seller_price,
):
    keys = [
        AccountMeta(pubkey=pool_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=escrow_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=long_token_mint_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=short_token_mint_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=buyer, is_signer=True, is_writable=False),
        AccountMeta(pubkey=seller, is_signer=True, is_writable=False),
        AccountMeta(pubkey=buyer_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=seller_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=buyer_long_token_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=buyer_short_token_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=seller_long_token_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=seller_short_token_account, is_signer=False, is_writable=True),
        AccountMeta(pubkey=escrow_authority_account, is_signer=False, is_writable=False),
        AccountMeta(pubkey=token_account, is_signer=False, is_writable=False),
    ]
    data = struct.pack("<BQQQ", 1, size, buyer_price, seller_price)
    return TransactionInstruction(keys=keys, program_id=PublicKey(BETTING_POOL_PROGRAM_ID), data=data)


class BettingPool():

    def __init__(self, cfg):
        self.private_key = list(base58.b58decode(cfg["PRIVATE_KEY"]))[:32]
        self.public_key = cfg["PUBLIC_KEY"]
        self.cipher = Fernet(cfg["DECRYPTION_KEY"])


    def initialize(self, api_endpoint, escrow_mint, decimals=2, skip_confirmation=True):
        msg = ""
        try:
            # Initalize Clinet
            client = Client(api_endpoint)
            msg += "Initialized client"
            # Create account objects
            source_account = Account(self.private_key)
            pool = Account()
            long_escrow = Account()
            short_escrow = Account()
            long_mint = Account()
            short_mint = Account()
            # List non-derived accounts
            pool_account = pool.public_key()
            escrow_mint_account = PublicKey(escrow_mint)
            escrow_account = long_escrow.public_key()
            long_token_mint_account = long_mint.public_key()
            short_token_mint_account = short_mint.public_key()
            mint_authority_account = source_account.public_key()
            update_authority_account = source_account.public_key()
            token_account = PublicKey(TOKEN_PROGRAM_ID)
            system_account = PublicKey(SYSTEM_PROGRAM_ID)
            rent_account = PublicKey(SYSVAR_RENT_ID)
            msg += " | Gathered accounts"
            # List signers
            signers = [source_account, long_mint, short_mint, long_escrow, short_escrow, pool]
            # Start transaction
            tx = Transaction()
            # Create Token Metadata
            init_betting_pool_ix =  initialize_betting_pool_instruction(
                pool_account,
                escrow_mint_account,
                escrow_account,
                long_token_mint_account,
                short_token_mint_account,
                mint_authority_account,
                update_authority_account,
                token_account,
                system_account,
                rent_account,
                decimals,
            )
            tx = tx.add(init_betting_pool_ix)
            msg += f" | Creating betting pool"
            # Send request
            try:
                response = client.send_transaction(tx, *signers, opts=types.TxOpts(skip_confirmation=skip_confirmation))
                return json.dumps(
                    {
                        'status': HTTPStatus.OK,
                        'betting_pool': str(pool_account),
                        'msg': msg + f" | Successfully created betting pool {str(pool_account)}",
                        'tx': response.get('result') if skip_confirmation else response['result']['transaction']['signatures'],
                    }
                )
            except Exception as e:
                msg += f" | ERROR: Encountered exception while attempting to send transaction: {e}"
                raise(e)
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': msg,
                }
            )    

    def trade(self, api_endpoint, pool_account, buyer_encrypted_private_key, seller_encrypted_private_key, size, buyer_price, seller_price, skip_confirmation=True):
        msg = ""
        try:
            client = Client(api_endpoint)
            msg += "Initialized client"
            # Create account objects
            buyer_private_key = list(self.cipher.decrypt(buyer_encrypted_private_key))
            seller_private_key = list(self.cipher.decrypt(seller_encrypted_private_key))
            assert(len(buyer_private_key) == 32)
            assert(len(seller_private_key) == 32)
            source_account = Account(self.private_key)
            buyer = Account(buyer_private_key)
            seller = Account(seller_private_key)

            # Signers
            signers = [buyer, seller, source_account]

            pool = self.load_betting_pool(api_endpoint, pool_account)
            # List non-derived accounts
            pool_account = PublicKey(pool_account) 
            escrow_account = PublicKey(pool["escrow"]) 
            escrow_mint_account = PublicKey(pool["escrow_mint"]) 
            long_token_mint_account = PublicKey(pool["long_mint"]) 
            short_token_mint_account = PublicKey(pool["short_mint"]) 
            buyer_account = buyer.public_key()
            seller_account = seller.public_key()
            token_account = PublicKey(TOKEN_PROGRAM_ID)
            escrow_owner_account = PublicKey.find_program_address(
                [bytes(long_token_mint_account), bytes(short_token_mint_account), bytes(token_account), bytes(PublicKey(BETTING_POOL_PROGRAM_ID))],
                PublicKey(BETTING_POOL_PROGRAM_ID),
            )[0]

            tx = Transaction()

            accts = set()
            atas = []
            for acct in [buyer_account, seller_account]:
                acct_atas = []
                for mint_account in (long_token_mint_account, short_token_mint_account, escrow_mint_account):
                    token_pda_address = PublicKey.find_program_address(
                        [bytes(acct), bytes(token_account), bytes(mint_account)],
                        PublicKey(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID),
                    )[0]
                    previously_seen = str(token_pda_address) in accts
                    accts.add(str(token_pda_address))
                    associated_token_account_info = client.get_account_info(token_pda_address)
                    account_info = associated_token_account_info['result']['value']
                    if account_info is not None: 
                        account_state = ACCOUNT_LAYOUT.parse(base64.b64decode(account_info['data'][0])).state
                    else:
                        account_state = 0
                    if account_state == 0 and not previously_seen:
                        msg += f" | Creating PDA: {token_pda_address}"
                        associated_token_account_ix = create_associated_token_account_instruction(
                            associated_token_account=token_pda_address,
                            payer=source_account.public_key(),
                            wallet_address=acct,
                            token_mint_address=mint_account,
                        )
                        tx = tx.add(associated_token_account_ix)
                    else:
                        msg += f" | Fetched PDA: {token_pda_address}"
                    acct_atas.append(token_pda_address)
                atas.append(acct_atas)
            trade_ix = trade_instruction(
                pool_account,
                escrow_account,
                long_token_mint_account,
                short_token_mint_account,
                buyer_account,
                seller_account,
                atas[0][2],
                atas[1][2],
                atas[0][0],
                atas[0][1],
                atas[1][0],
                atas[1][1],
                escrow_owner_account,
                token_account,
                int(size),
                int(buyer_price),
                int(seller_price),
            )
            tx = tx.add(trade_ix)
            # Send request
            try:
                response = client.send_transaction(tx, *signers, opts=types.TxOpts(skip_confirmation=skip_confirmation))
                return json.dumps(
                    {
                        'status': HTTPStatus.OK,
                        'msg': msg + f" | Trade successful",
                        'tx': response.get('result') if skip_confirmation else response['result']['transaction']['signatures'],
                    }
                )
            except Exception as e:
                msg += f" | ERROR: Encountered exception while attempting to send transaction: {e}"
                raise(e)
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': msg,
                }
            )    

    def load_betting_pool(self, api_endpoint, pool_account):
        client = Client(api_endpoint)
        try:
            pool_data = base64.b64decode(client.get_account_info(pool_account)['result']['value']['data'][0])
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': str(e),
                }
            )
        pubkey = 'B' * 32
        raw_bytes = struct.unpack(f"<BQ?{pubkey}{pubkey}{pubkey}{pubkey}{pubkey}", pool_data)
        i = 0
        pool = {}
        pool["decimals"] = raw_bytes[i] 
        i += 1
        pool["circulation"] = raw_bytes[i] 
        i += 1
        pool["settled"] = raw_bytes[i] 
        i += 1
        pool["escrow_mint"] = base58.b58encode(bytes(raw_bytes[i:i+32])).decode('ascii')
        i += 32
        pool["escrow"] = base58.b58encode(bytes(raw_bytes[i:i+32])).decode('ascii')
        i += 32
        pool["long_mint"] = base58.b58encode(bytes(raw_bytes[i:i+32])).decode('ascii')
        i += 32
        pool["short_mint"] = base58.b58encode(bytes(raw_bytes[i:i+32])).decode('ascii')
        i += 32
        pool["winning_side"] = base58.b58encode(bytes(raw_bytes[i:i+32])).decode('ascii')
        i += 32
        return pool

    def wallet(self):
        """ Generate a wallet and return the address and private key. """
        account = Account()
        pub_key = account.public_key() 
        private_key = list(account.secret_key()[:32])
        return json.dumps(
            {
                'address': str(pub_key),
                'private_key': private_key
            }
        )

    def topup(self, api_endpoint, to, amount=None, skip_confirmation=True):
        """
        Send a small amount of native currency to the specified wallet to handle gas fees. Return a status flag of success or fail and the native transaction data.
        """
        msg = ""
        try:
            # Connect to the api_endpoint
            client = Client(api_endpoint)
            msg += "Initialized client"
            # List accounts 
            sender_account = Account(self.private_key)
            dest_account = PublicKey(to)
            msg += " | Gathered accounts"
            # List signers
            signers = [sender_account]
            # Start transaction
            tx = Transaction()
            # Determine the amount to send 
            try:
                if amount is None:
                    min_rent_reseponse = client.get_minimum_balance_for_rent_exemption(ACCOUNT_LAYOUT.sizeof())
                    lamports = min_rent_reseponse["result"]
                else:
                    lamports = int(amount)
                msg += f" | Fetched lamports: {lamports * 1e-9} SOL"
            except Exception as e:
                msg += " | ERROR: couldn't process lamports" 
                raise(e)
            # Generate transaction
            transfer_ix = transfer(TransferParams(from_pubkey=sender_account.public_key(), to_pubkey=dest_account, lamports=lamports))
            tx = tx.add(transfer_ix)
            msg += f" | Transferring funds"
            # Send request
            try:
                response = client.send_transaction(tx, *signers, opts=types.TxOpts(skip_confirmation=skip_confirmation))
                return json.dumps(
                    {
                        'status': HTTPStatus.OK,
                        'msg': f"Successfully sent {lamports * 1e-9} SOL to {to}",
                        'tx': response.get('result') if skip_confirmation else response['result']['transaction']['signatures'],
                    }
                )
            except Exception as e:
                msg += f" | ERROR: Encountered exception while attempting to send transaction: {e}"
                raise(e)
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': msg,
                }
            )

    def mint_to(self, api_endpoint, pool_account, dest, amount, skip_confirmation=True):
        msg = ""
        try:
            client = Client(api_endpoint)
            msg += "Initialized client"
            # Create account objects
            source_account = Account(self.private_key)
            signers = [source_account]
            pool = self.load_betting_pool(api_endpoint, pool_account)
            # List non-derived accounts
            pool_account = PublicKey(pool_account) 
            dest_account = PublicKey(dest)
            escrow_mint_account = PublicKey(pool["escrow_mint"]) 
            mint_authority_account = source_account.public_key()
            payer_account = source_account.public_key()
            token_account = PublicKey(TOKEN_PROGRAM_ID)

            tx = Transaction()

            accts = set()
            for mint_account in [escrow_mint_account]:
                token_pda_address = PublicKey.find_program_address(
                    [bytes(dest_account), bytes(token_account), bytes(mint_account)],
                    PublicKey(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID),
                )[0]
                if str(token_pda_address) in accts:
                    continue
                accts.add(str(token_pda_address))
                associated_token_account_info = client.get_account_info(token_pda_address)
                account_info = associated_token_account_info['result']['value']
                if account_info is not None: 
                    account_state = ACCOUNT_LAYOUT.parse(base64.b64decode(account_info['data'][0])).state
                else:
                    account_state = 0
                if account_state == 0:
                    msg += f" | Creating PDA: {token_pda_address}"
                    associated_token_account_ix = create_associated_token_account_instruction(
                        associated_token_account=token_pda_address,
                        payer=payer_account, # signer
                        wallet_address=dest_account,
                        token_mint_address=mint_account,
                    )
                    tx = tx.add(associated_token_account_ix)
                mint_to_ix = mint_to(
                    MintToParams(
                        program_id=token_account,
                        mint=mint_account,
                        dest=token_pda_address,
                        mint_authority=mint_authority_account,
                        amount=int(amount),
                        signers=[mint_authority_account],
                    )
                )
                tx = tx.add(mint_to_ix) 
            # Send request
            try:
                response = client.send_transaction(tx, *signers, opts=types.TxOpts(skip_confirmation=skip_confirmation))
                return json.dumps(
                    {
                        'status': HTTPStatus.OK,
                        'msg': msg + f" | Success",
                        'tx': response.get('result') if skip_confirmation else response['result']['transaction']['signatures'],
                    }
                )
            except Exception as e:
                msg += f" | ERROR: Encountered exception while attempting to send transaction: {e}"
                raise(e)
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': msg,
                }
            )     

    def create_mint(self, api_endpoint, skip_confirmation=True):
        msg = ''
        try:
            client = Client(api_endpoint)
            msg += "Initialized client"
            # List non-derived accounts
            source_account = Account(self.private_key)
            mint_account = Account()
            token_account = PublicKey(TOKEN_PROGRAM_ID)
            msg += " | Gathered accounts"
            # List signers
            signers = [source_account, mint_account]
            # Start transaction
            tx = Transaction()
            # Get the minimum rent balance for a mint account
            try:
                min_rent_reseponse = client.get_minimum_balance_for_rent_exemption(MINT_LAYOUT.sizeof())
                lamports = min_rent_reseponse["result"]
                msg += f" | Fetched minimum rent exemption balance: {lamports * 1e-9} SOL"
            except Exception as e:
                msg += " | ERROR: Failed to receive min balance for rent exemption"
                raise(e)
            # Generate Mint 
            create_mint_account_ix = create_account(
                CreateAccountParams(
                    from_pubkey=source_account.public_key(),
                    new_account_pubkey=mint_account.public_key(),
                    lamports=lamports,
                    space=MINT_LAYOUT.sizeof(),
                    program_id=token_account,
                )
            )
            tx = tx.add(create_mint_account_ix)
            msg += f" | Creating mint account {str(mint_account.public_key())} with {MINT_LAYOUT.sizeof()} bytes"
            initialize_mint_ix = initialize_mint(
                InitializeMintParams(
                    decimals=0,
                    program_id=token_account,
                    mint=mint_account.public_key(),
                    mint_authority=source_account.public_key(),
                    freeze_authority=source_account.public_key(),
                )
            )
            tx = tx.add(initialize_mint_ix)
            try:
                response = client.send_transaction(tx, *signers, opts=types.TxOpts(skip_confirmation=skip_confirmation))
                return json.dumps(
                    {
                        'status': HTTPStatus.OK,
                        'mint': str(mint_account.public_key()),
                        'msg': f"Successfully created mint {str(mint_account.public_key())}",
                        'tx': response.get('result') if skip_confirmation else response['result']['transaction']['signatures'],
                    }
                )
            except Exception as e:
                msg += f" | ERROR: Encountered exception while attempting to send transaction: {e}"
                raise(e)
        except Exception as e:
            return json.dumps(
                {
                    'status': HTTPStatus.BAD_REQUEST,
                    'msg': msg,
                }
            )         