use fadroma::auth::{assert_admin, save_admin};
use fadroma::platform::{Extern, Storage, Api, Querier, Env, HandleResponse, StdResult, HumanAddr};

pub fn change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_owner: String,
) -> StdResult<HandleResponse> {
    assert_admin(deps, &env)?;
    save_admin(deps, &HumanAddr(new_owner))?;
    Ok(HandleResponse::default())
}
