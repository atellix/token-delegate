import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { TokenDelegate } from '../target/types/token_delegate';

describe('token-delegate', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.TokenDelegate as Program<TokenDelegate>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
