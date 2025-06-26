#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

// Storage key implementation for Soroban replacing Solidity's nested mappings
// Replaces Solidity's mapping(address => mapping(uint256 => uint256)) private _balance;
// Uses keys/variables that Soroban serializes automatically
#[contracttype]
pub enum DataKey {
    // Contract core data
    Admin,

    // Asset ID counter replacing id_counter from various registry implementations
    NextAssetId,

    // Balance and supply tracking
    // Replaces mapping(address => mapping(uint256 => uint256)) private _balance;
    Balance(Address, u64), // owner -> asset_id -> balance

    // Replaces mapping(uint256 => uint256) private _totalSupply;
    AssetSupply(u64), // asset_id -> total_supply

    // Ownership tracking
    // Replaces complex tree structures from RegistryNestedTree
    // Avoids unlimited Vec growth through simple boolean flags
    AssetOwnerExists(u64, Address), // asset_id -> owner -> bool
    OwnerAssetExists(Address, u64), // owner -> asset_id -> bool
    AssetOwnerCount(u64),           // asset_id -> number_of_owners

    // Funding distributions
    // New addition - replaces complex tree queries from Solidity
    // Enables efficient iteration for future funding/voting systems
    AssetOwnersList(u64), // asset_id -> Vec<Address> (owners with balance > 0)
    OwnerAssetsList(Address), // owner -> Vec<u64> (assets owned with balance > 0)

    // Authorization system
    // Simplification of AllowancesNestedMap from Solidity
    // Maintains ERC1155 compatibility but with simpler storage
    OperatorApproval(Address, Address), // owner -> operator -> approved_for_all
    TokenAllowance(Address, Address, u64), // owner -> operator -> asset_id -> allowance

    // Metadata support
    // Replaces mapping(uint256 => string) assetURIs; from Solidity
    AssetURI(u64), // asset_id -> metadata_uri
    ContractURI,   // global contract metadata

    // Asset management
    // New functionality - tracking who created each asset
    AssetCreator(u64), // asset_id -> creator_address
}

#[contract]
pub struct FractionalizationContract;

#[contractimpl]
impl FractionalizationContract {
    /// Contract initialization replacing function initialize(string memory uri_) public virtual initializer
    /// Simplification: Removes initial URI, focuses only on admin setup
    /// Admin setup is mandatory (Solidity allowed deployment without defined admin)
    pub fn initialize(env: Env, admin: Address) {
        // Reentrancy protection (similar to initializer modifier from Solidity)
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        // Mandatory authorization verification in Soroban
        admin.require_auth();

        // Set admin - replaces _admin = msg.sender; from Solidity
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Initialize asset ID counter - replaces id_counter = 1; from registries
        env.storage().instance().set(&DataKey::NextAssetId, &1u64);

        // Emit initialization event - similar to Solidity events but simpler
        env.events().publish((symbol_short!("init"),), (admin,));
    }

    /// Token minting replacing function mint(uint256 numTokens) public virtual noReentrancy delegateOnly returns (uint256)
    /// Simplification: Removes reentrancy guard (Soroban has built-in protections)
    /// Change: Mint to specific address instead of msg.sender
    /// Returns: asset_id instead of returning via event
    pub fn mint(env: Env, to: Address, num_tokens: u64) -> u64 {
        // Admin verification - replaces delegateOnly modifier from Solidity
        Self::require_admin_auth(env.clone());

        // Basic validation - similar to Solidity but more explicit
        if num_tokens == 0 {
            panic!("Cannot mint 0 tokens");
        }

        // Get next asset ID - replaces id_counter++ from Solidity
        let asset_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextAssetId)
            .unwrap_or(1);

        // Increment for next mint
        env.storage()
            .instance()
            .set(&DataKey::NextAssetId, &(asset_id + 1));

        // Set owner balance - replaces registry system from Solidity
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone(), asset_id), &num_tokens);

        // Set total supply - similar to _totalSupply[tokenId] = n_tokens;
        env.storage()
            .persistent()
            .set(&DataKey::AssetSupply(asset_id), &num_tokens);

        // Creator tracking - new functionality vs Solidity
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        env.storage()
            .persistent()
            .set(&DataKey::AssetCreator(asset_id), &admin);

        // Efficient ownership tracking
        // Replaces complex tree structures from RegistryNestedTree
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerExists(asset_id, to.clone()), &true);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerAssetExists(to.clone(), asset_id), &true);
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerCount(asset_id), &1u32);

        // List maintenance for future systems
        // New functionality - enables efficient queries for funding/voting
        Self::add_owner_to_asset(&env, asset_id, to.clone());
        Self::add_asset_to_owner(&env, to.clone(), asset_id);

        // Emit event - similar to TransferSingle from ERC1155 but simplified
        env.events()
            .publish((symbol_short!("mint"),), (to.clone(), asset_id, num_tokens));

        asset_id
    }

    /// Multiple recipient minting from Solidity's dynamicMint() - simplified version
    /// Allows minting to multiple recipients of an existing asset
    /// Validation: asset_id must exist (cannot create new assets)
    pub fn mint_to(env: Env, asset_id: u64, recipients: Vec<Address>, amounts: Vec<u64>) {
        // Admin verification
        Self::require_admin_auth(env.clone());

        // Asset ID validation - prevents creation of new assets via mint_to
        if asset_id == 0 {
            panic!("Asset ID cannot be 0 - use mint() to create new assets");
        }

        // Verify asset exists - new functionality vs Solidity
        if !Self::asset_exists(env.clone(), asset_id) {
            panic!("Asset does not exist");
        }

        // Input validations - similar to Solidity
        if recipients.len() != amounts.len() {
            panic!("Recipients and amounts length mismatch");
        }

        if recipients.len() == 0 {
            panic!("No recipients specified");
        }

        let mut total_minted = 0u64;
        let mut owner_count = Self::get_asset_owner_count(env.clone(), asset_id);

        // Process each recipient
        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            if amount == 0 {
                panic!("Cannot mint 0 tokens");
            }

            // Update balance (may add to existing balance)
            let current_balance = Self::balance_of(env.clone(), recipient.clone(), asset_id);
            env.storage().persistent().set(
                &DataKey::Balance(recipient.clone(), asset_id),
                &(current_balance + amount),
            );

            // Track new ownership only if didn't have tokens before
            if current_balance == 0 {
                env.storage().persistent().set(
                    &DataKey::AssetOwnerExists(asset_id, recipient.clone()),
                    &true,
                );
                env.storage().persistent().set(
                    &DataKey::OwnerAssetExists(recipient.clone(), asset_id),
                    &true,
                );
                owner_count += 1;

                // Add to lists for future queries
                Self::add_owner_to_asset(&env, asset_id, recipient.clone());
                Self::add_asset_to_owner(&env, recipient.clone(), asset_id);
            }

            total_minted += amount;

            // Emitir evento por recipient
            env.events().publish(
                (symbol_short!("mint_to"),),
                (recipient.clone(), asset_id, amount),
            );
        }

        // Atualizar supply total
        let current_supply = Self::asset_supply(env.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::AssetSupply(asset_id),
            &(current_supply + total_minted),
        );

        // Atualizar count de owners
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerCount(asset_id), &owner_count);
    }

    /// REFACTOR: function balanceOf(address account, uint256 id) external view returns (uint256)
    /// Implementação direta - sem overhead das árvores do Solidity
    pub fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64 {
        // Acesso direto ao storage - muito mais simples que as árvores do RegistryNestedTree
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner, asset_id))
            .unwrap_or(0) // Retorna 0 se não existir (como no Solidity)
    }

    /// REFACTOR: function balanceOfBatch() do ERC1155
    /// Implementação mantida para compatibilidade
    pub fn balance_of_batch(env: Env, owners: Vec<Address>, asset_ids: Vec<u64>) -> Vec<u64> {
        // Validação similar ao Solidity
        if owners.len() != asset_ids.len() {
            panic!("Owners and asset_ids length mismatch");
        }

        let mut balances = Vec::new(&env);
        // Iterar e obter balances individuais
        for i in 0..owners.len() {
            let owner = owners.get(i).unwrap();
            let asset_id = asset_ids.get(i).unwrap();
            let balance = Self::balance_of(env.clone(), owner, asset_id);
            balances.push_back(balance);
        }

        balances
    }

    /// NOVA FUNÇÃO: Transfer simples (owner transfere seus próprios tokens)
    /// Simplificação vs safeTransferFrom do Solidity
    pub fn transfer(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
        // Verificação de autorização obrigatória
        from.require_auth();
        // Delegar para lógica interna
        Self::transfer_internal(env, from, to, asset_id, amount);
    }

    /// REFACTOR: function safeTransferFrom() do ERC1155
    /// Simplificação: Remove callback de segurança (será implementado em layer superior)
    /// Mantém: Sistema de allowances e autorização
    pub fn transfer_from(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        asset_id: u64,
        amount: u64,
    ) {
        // === VERIFICAÇÃO DE AUTORIZAÇÃO ===
        // Simplificação vs _verifyTransaction do AllowancesNestedMap
        if operator != from {
            // Verificar se tem approval for all
            let approved_for_all =
                Self::is_approved_for_all(env.clone(), from.clone(), operator.clone());

            if !approved_for_all {
                // Verificar allowance específica para este token
                let allowance: u64 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::TokenAllowance(
                        from.clone(),
                        operator.clone(),
                        asset_id,
                    ))
                    .unwrap_or(0);

                if allowance < amount {
                    panic!("Insufficient allowance");
                }

                // Decrementar allowance - similar ao updateAllowanceRecords do Solidity
                env.storage().persistent().set(
                    &DataKey::TokenAllowance(from.clone(), operator.clone(), asset_id),
                    &(allowance - amount),
                );
            }
        } else {
            // Se operator == from, verificar autorização direta
            from.require_auth();
        }

        // Executar transfer
        Self::transfer_internal(env, from, to, asset_id, amount);
    }

    /// REFACTOR: function _transferFrom() dos vários registries do Solidity
    /// Simplificação: Lógica unificada sem overhead das estruturas de árvore
    /// Adição: Manutenção automática das listas de owners
    fn transfer_internal(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
        // Validações básicas
        if amount == 0 {
            panic!("Cannot transfer 0 tokens");
        }

        if from == to {
            panic!("Cannot transfer to self");
        }

        // Obter balances atuais - acesso direto vs queries complexas do Solidity
        let from_balance = Self::balance_of(env.clone(), from.clone(), asset_id);
        let to_balance = Self::balance_of(env.clone(), to.clone(), asset_id);

        if from_balance < amount {
            panic!("Insufficient balance");
        }

        // Calcular novos balances
        let new_from_balance = from_balance - amount;
        let new_to_balance = to_balance + amount;

        // Atualizar balances no storage
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone(), asset_id), &new_from_balance);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone(), asset_id), &new_to_balance);

        // === TRACKING DE NOVA OWNERSHIP ===
        // Funcionalidade nova vs Solidity - manutenção automática de listas
        if to_balance == 0 {
            // Recipient é novo owner deste asset
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnerExists(asset_id, to.clone()), &true);
            env.storage()
                .persistent()
                .set(&DataKey::OwnerAssetExists(to.clone(), asset_id), &true);

            // Incrementar count de owners
            let owner_count = Self::get_asset_owner_count(env.clone(), asset_id);
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count + 1));

            // Adicionar às listas para queries eficientes
            Self::add_owner_to_asset(&env, asset_id, to.clone());
            Self::add_asset_to_owner(&env, to.clone(), asset_id);
        }

        // === CLEANUP DE OWNERSHIP ===
        // Remover sender das listas se ficou com balance 0
        if new_from_balance == 0 {
            Self::remove_owner_from_asset(&env, asset_id, from.clone());
            Self::remove_asset_from_owner(&env, from.clone(), asset_id);

            // Decrementar count de owners
            let owner_count = Self::get_asset_owner_count(env.clone(), asset_id);
            if owner_count > 0 {
                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count - 1));
            }
        }

        // Emitir evento de transfer
        env.events().publish(
            (symbol_short!("transfer"),),
            (from.clone(), to.clone(), asset_id, amount),
        );
    }

    /// REFACTOR: function safeBatchTransferFrom() do ERC1155
    /// Simplificação: Remove callback de segurança, mantém lógica de autorização
    pub fn batch_transfer_from(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        asset_ids: Vec<u64>,
        amounts: Vec<u64>,
    ) {
        // Validação de arrays
        if asset_ids.len() != amounts.len() {
            panic!("Asset IDs and amounts length mismatch");
        }

        // Executar transfers individuais - cada um com suas próprias validações
        for i in 0..asset_ids.len() {
            let asset_id = asset_ids.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            Self::transfer_from(
                env.clone(),
                operator.clone(),
                from.clone(),
                to.clone(),
                asset_id,
                amount,
            );
        }
    }

    /// REFACTOR: function setApprovalForAll() do ERC1155
    /// Mantida funcionalidade completa para compatibilidade
    pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
        // Verificação de autorização
        owner.require_auth();

        // Armazenar approval - storage direto vs nested mappings do Solidity
        env.storage().persistent().set(
            &DataKey::OperatorApproval(owner.clone(), operator.clone()),
            &approved,
        );

        // Emitir evento
        env.events()
            .publish((symbol_short!("approval"),), (owner, operator, approved));
    }

    /// REFACTOR: function isApprovedForAll() do ERC1155
    /// Implementação direta
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::OperatorApproval(owner, operator))
            .unwrap_or(false)
    }

    /// REFACTOR: function approval() do Solidity (AllowancesNestedMap)
    /// Renomeado para approve para compatibilidade ERC20/ERC1155
    pub fn approve(env: Env, owner: Address, operator: Address, asset_id: u64, amount: u64) {
        owner.require_auth();

        // Armazenar allowance específica
        env.storage().persistent().set(
            &DataKey::TokenAllowance(owner.clone(), operator.clone(), asset_id),
            &amount,
        );

        // Emitir evento
        env.events().publish(
            (symbol_short!("approve"),),
            (owner, operator, asset_id, amount),
        );
    }

    /// REFACTOR: function getAllowance() do AllowancesNestedMap
    /// Renomeado para allowance para compatibilidade padrão
    pub fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::TokenAllowance(owner, operator, asset_id))
            .unwrap_or(0)
    }

    /// REFACTOR: function assetSupply(uint256 assetId) external view returns (uint256)
    /// Implementação direta vs cálculos complexos dos registries
    pub fn asset_supply(env: Env, asset_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetSupply(asset_id))
            .unwrap_or(0)
    }

    /// NOVA FUNÇÃO: Count de owners por asset
    /// Substitui queries caras das estruturas de árvore do Solidity
    pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnerCount(asset_id))
            .unwrap_or(0)
    }

    /// NOVA FUNÇÃO: Verificação rápida de ownership
    /// Substitui iterações complexas do RegistryNestedTree
    pub fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnerExists(asset_id, owner))
            .unwrap_or(false)
    }

    /// NOVA FUNÇÃO: Verificação se owner tem algum asset
    /// Funcionalidade auxiliar para queries
    pub fn has_assets(env: Env, owner: Address, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerAssetExists(owner, asset_id))
            .unwrap_or(false)
    }

    /// REFACTOR: function assetOwners(uint256 tokenId) external view returns (address[] memory)
    /// Simplificação: Lista direta vs iteração complexa das árvores
    /// Otimização: Mantida em sync automaticamente vs cálculo on-demand
    pub fn asset_owners(env: Env, asset_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(&env))
    }

    /// REFACTOR: function addressAssets(address owner) external view returns (uint256[] memory)
    /// Simplificação similar à função acima
    pub fn owner_assets(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// NOVA FUNÇÃO: Próximo asset ID a ser atribuído
    /// Funcionalidade auxiliar para front-ends
    pub fn next_asset_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextAssetId)
            .unwrap_or(1)
    }

    /// NOVA FUNÇÃO: Verificação de existência de asset
    /// Substitui verificações complexas do Solidity
    pub fn asset_exists(env: Env, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::AssetSupply(asset_id))
    }

    /// REFACTOR: function setUri(uint256 _tokenId, string calldata uri_) public
    /// Adição: Verificação de autorização (admin ou creator)
    /// Mudança: Caller explícito vs msg.sender implícito
    pub fn set_asset_uri(env: Env, caller: Address, asset_id: u64, uri: String) {
        caller.require_auth();

        // Verificar se asset existe
        if !Self::asset_exists(env.clone(), asset_id) {
            panic!("Asset does not exist");
        }

        // Verificação de autorização - apenas admin ou creator do asset
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let creator: Address = env
            .storage()
            .persistent()
            .get(&DataKey::AssetCreator(asset_id))
            .unwrap();

        if caller != admin && caller != creator {
            panic!("Not authorized to set URI");
        }

        // Armazenar URI
        env.storage()
            .persistent()
            .set(&DataKey::AssetURI(asset_id), &uri);

        // Emitir evento
        env.events()
            .publish((symbol_short!("uri"),), (asset_id, uri));
    }

    /// REFACTOR: function uri(uint256 _tokenId) public view returns (string memory)
    /// Implementação direta
    pub fn asset_uri(env: Env, asset_id: u64) -> Option<String> {
        env.storage().persistent().get(&DataKey::AssetURI(asset_id))
    }

    /// NOVA FUNÇÃO: URI de nível de contrato
    /// Funcionalidade adicional para metadata global
    pub fn set_contract_uri(env: Env, caller: Address, uri: String) {
        Self::require_admin_auth(env.clone());
        caller.require_auth();

        env.storage().persistent().set(&DataKey::ContractURI, &uri);
    }

    /// NOVA FUNÇÃO: Obter URI de contrato
    pub fn contract_uri(env: Env) -> Option<String> {
        env.storage().persistent().get(&DataKey::ContractURI)
    }

    /// REFACTOR: function getAdmin() public view returns (address)
    /// Implementação direta
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// NOVA FUNÇÃO: Obter creator de asset
    /// Tracking adicional vs Solidity original
    pub fn get_asset_creator(env: Env, asset_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetCreator(asset_id))
    }

    /// NOVA FUNÇÃO: Transfer de role de admin
    /// Funcionalidade de governança básica
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        Self::require_admin_auth(env.clone());
        current_admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events()
            .publish((symbol_short!("admin"),), (current_admin, new_admin));
    }

    /// REFACTOR: modifier onlyAdmin do Solidity
    /// Convertido para função helper
    fn require_admin_auth(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
    }

    // === FUNÇÕES INTERNAS DE GESTÃO DE LISTAS ===
    // ADIÇÃO NOVA: Substitui as estruturas complexas de árvore do RegistryNestedTree
    // Permite queries eficientes para sistemas de funding e voting futuros

    /// Adicionar owner à lista de owners de um asset
    /// Mantém lista atualizada para queries rápidas
    fn add_owner_to_asset(env: &Env, asset_id: u64, owner: Address) {
        let mut owners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(env));

        // Verificar se owner já está na lista (evitar duplicatas)
        let mut found = false;
        for i in 0..owners.len() {
            if owners.get(i).unwrap() == owner {
                found = true;
                break;
            }
        }

        // Adicionar apenas se não existir
        if !found {
            owners.push_back(owner.clone());
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnersList(asset_id), &owners);
        }
    }

    /// Remover owner da lista quando balance = 0
    /// Manutenção automática vs manual cleanup do Solidity
    fn remove_owner_from_asset(env: &Env, asset_id: u64, owner: Address) {
        let owners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(env));

        // Filtrar owner a remover
        let mut new_owners = Vec::new(env);
        for i in 0..owners.len() {
            let current_owner = owners.get(i).unwrap();
            if current_owner != owner {
                new_owners.push_back(current_owner);
            }
        }

        // Atualizar lista
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnersList(asset_id), &new_owners);
    }

    /// Adicionar asset à lista de assets de um owner
    /// Funcionalidade simétrica para queries bidirecionais
    fn add_asset_to_owner(env: &Env, owner: Address, asset_id: u64) {
        let mut assets: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner.clone()))
            .unwrap_or(Vec::new(env));

        // Verificar duplicatas
        let mut found = false;
        for i in 0..assets.len() {
            if assets.get(i).unwrap() == asset_id {
                found = true;
                break;
            }
        }

        // Adicionar se novo
        if !found {
            assets.push_back(asset_id);
            env.storage()
                .persistent()
                .set(&DataKey::OwnerAssetsList(owner), &assets);
        }
    }

    /// Remover asset da lista de owner quando balance = 0
    /// Cleanup automático para manter listas consistentes
    fn remove_asset_from_owner(env: &Env, owner: Address, asset_id: u64) {
        let assets: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner.clone()))
            .unwrap_or(Vec::new(env));

        // Filtrar asset a remover
        let mut new_assets = Vec::new(env);
        for i in 0..assets.len() {
            let current_asset = assets.get(i).unwrap();
            if current_asset != asset_id {
                new_assets.push_back(current_asset);
            }
        }

        // Atualizar lista do owner
        env.storage()
            .persistent()
            .set(&DataKey::OwnerAssetsList(owner), &new_assets);
    }
}

mod test;
