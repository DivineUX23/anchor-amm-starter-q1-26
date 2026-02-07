import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmmStarterQ126 } from "../target/types/anchor_amm_starter_q1_26";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction, createMint, mintTo, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { expect } from "chai";

describe("anchor-amm-starter-q1-26", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorAmmStarterQ126 as Program<AnchorAmmStarterQ126>;
  const connection = provider.connection;

  const initializer = provider.wallet;
  const user = Keypair.generate();

  let configPda: PublicKey;
  let mintX: PublicKey;
  let mintY: PublicKey;
  let vaultX: PublicKey;
  let vaultY: PublicKey;
  let mintLp: PublicKey;
  let userLp: PublicKey;
  let mintXAta: PublicKey;
  let mintYAta: PublicKey;
  let Bump: number;

  const depositAmount = 100;
  const fee = 10;
  const maxX = 100;
  const maxY = 100;

  before(async () => {
    
    await connection.requestAirdrop(user.publicKey, 5_000_000_000); // 5 SOL
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create mints (decimals=0 for simplicity)
    mintX = await createMint(provider.connection, provider.wallet.payer, initializer.publicKey, null, 0);
    mintY = await createMint(provider.connection, provider.wallet.payer, initializer.publicKey, null, 0);

    // Create ATAs and mint tokens
    mintXAta = getAssociatedTokenAddressSync(mintX, user.publicKey);
    const mintXAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(provider.wallet.publicKey, mintXAta, user.publicKey, mintX)
    );
    await provider.sendAndConfirm(mintXAtaTx);
    await mintTo(provider.connection, provider.wallet.payer, mintX, mintXAta, provider.wallet.payer, depositAmount * 2);

    mintYAta = getAssociatedTokenAddressSync(mintY, user.publicKey);
    const mintYAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(provider.wallet.publicKey, mintYAta, user.publicKey, mintY)
    );
    await provider.sendAndConfirm(mintYAtaTx);
    await mintTo(provider.connection, provider.wallet.payer, mintY, mintYAta, provider.wallet.payer, depositAmount * 2);


    const seed1 = new anchor.BN(1111);
    [configPda, Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed1.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    mintLp = PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), configPda.toBuffer()],
      program.programId
    )[0];

    userLp = getAssociatedTokenAddressSync(
      mintLp,
      user.publicKey
    );

    vaultX = getAssociatedTokenAddressSync(
      mintX,
      configPda, true
    );

    vaultY = getAssociatedTokenAddressSync(
      mintY,
      configPda, true
    );
  })

  it("Is initialized!", async () => {
    const seed1 = new anchor.BN(1111);

    // Init
    await program.methods
      .initialize(seed1, fee, initializer.publicKey)
      .accountsStrict({
        initializer: initializer.publicKey,
        mintX: mintX,
        mintY: mintY,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        config: configPda,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const config = await program.account.config.fetch(configPda);
    expect(config.authority.toBase58()).to.equal(initializer.publicKey.toBase58());
    expect(config.mintX.toBase58()).to.equal(mintX.toBase58());
    expect(config.mintY.toBase58()).to.equal(mintY.toBase58());
    expect(config.fee).to.equal(fee);
    expect(config.configBump).to.equal(Bump);

  });

  it("Is deposited!", async () => {
    const seed1 = new anchor.BN(1111);

    await program.methods
      .deposit(new anchor.BN(maxX), new anchor.BN(maxY),  new anchor.BN(depositAmount))
      .accountsStrict({
        user: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        mintXAta: mintXAta,
        mintYAta: mintYAta,
        mintLp: mintLp,
        userLp: userLp,
        vaultX: vaultX,
        vaultY: vaultY,
        config: configPda,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const mintXInfo = await getAccount(provider.connection, vaultX);
    expect(mintXInfo.amount.toString()).to.equal(maxX.toString());

    const mintYInfo = await getAccount(provider.connection, vaultY);
    expect(mintYInfo.amount.toString()).to.equal(maxY.toString());

    const userLpInfo = await getAccount(provider.connection, userLp);
    expect(userLpInfo.amount.toString()).to.equal(depositAmount.toString());

  });



  it("Is Withdrawn!", async () => {
    const seed1 = new anchor.BN(1111);

    await program.methods
      .withdraw(new anchor.BN(maxX), new anchor.BN(maxY),  new anchor.BN(depositAmount))
      .accountsStrict({
        user: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        mintXAta: mintXAta,
        mintYAta: mintYAta,
        mintLp: mintLp,
        userLp: userLp,
        vaultX: vaultX,
        vaultY: vaultY,
        config: configPda,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const mintXInfo = await getAccount(provider.connection, mintXAta);
    expect(Number(mintXInfo.amount)).to.greaterThan(0);

    const mintYInfo = await getAccount(provider.connection, mintXAta);
    expect(Number(mintYInfo.amount)).to.greaterThan(0);

    const userLpInfo = await getAccount(provider.connection, userLp);
    expect(Number(userLpInfo.amount)).to.equal(0);

  });

});
