import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { GCStaking } from '../target/types/gc_staking';

describe('gc-staking', () => {
	// Configure the client to use the local cluster.
	anchor.setProvider(anchor.AnchorProvider.env());

	const program = anchor.workspace.BasicStaking as Program<GCStaking>;

	it('Is initialized!', async () => {
		// Add your test here.
		const tx = await program.methods.initialize().rpc();
		console.log('Your transaction signature', tx);
	});
});
