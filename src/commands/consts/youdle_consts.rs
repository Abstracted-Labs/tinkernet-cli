use crate::commands::youdle_staking_distribution::{GQLQuery, Variables};

pub const ONE_WITH_DECIMALS: u128 = 1_000_000_000_000;
pub const YOUDLE_DAO_ID: u32 = 0;
pub const YOUDLE_DAO_ADDRESS: &str = "i51CqF5bdj8rNEL4DXdYS6g4k7TT8sJK37JRHSqh58SP5zupa";
pub const KUSAMA_RMRK_GRAPHQL: &str = "https://gql-rmrk2-prod.graphcdn.app/";
pub const TINKERNET_OCIF_SQUID: &str = "https://squid.subsquid.io/ocif-squid/graphql";
pub const UNCLAIMED_CORE_QUERY: GQLQuery = GQLQuery {
    operationName: "unclaimed",
    variables: Variables {},
    query: "query unclaimed {
  cores(where: {coreId_eq: 0}) {
    totalUnclaimed
  }

stakers(where: {account_eq: \"i51CqF5bdj8rNEL4DXdYS6g4k7TT8sJK37JRHSqh58SP5zupa\"}) {
    totalUnclaimed
  }
}
",
};
pub const YOUDLES_QUERY: GQLQuery = GQLQuery {
    operationName: "tickets",
    variables: Variables {},
    query: "query tickets {
  	banners: nfts(
    where: {
    	collectionId: {
      	_eq: \"36af143c6012f6266b-YOUDLE_BRAND\"
      },
     	burned: { _eq: \"\" },

      metadata_name: {_eq: \"YoudleDAO Bannooooor\"},

    },

    order_by: {
    	id: asc
  	}
  ) {
    id
    rootowner
  }

  backgrounds: nfts(
    where: {
    	symbol: {
      	_eq: \"YOUDLEBACKGROUND\"
      },
     	burned: { _eq: \"\" },

      metadata_name: {_regex: \"Youdle Background #00(19|18)\"},

    },

    order_by: {
    	id: asc
  	}
  ) {
    id
    metadata_name
    rootowner
    owner
    parent { id }
  }

	eyes: nfts(
    where: {
    	symbol: {
      	_eq: \"YOUDLEEYES\"
      },
     	burned: { _eq: \"\" },

      metadata_name: {_eq: \"Youdle Eyes #0020\"},

    },

    order_by: {
    	id: asc
  	}
  ) {
    id
    rootowner
    owner
    parent { id }
  }

  youdles: nfts(
    where: {
    	collectionId: {
      	_eq: \"342f12106eab6d904c-YOUDLE\"
      },
     burned: { _eq: \"\" },

      _and: [
        {children: { collectionId: { _eq: \"342f12106eab6d904c-YOUDLEEYES\" } }}
        {children: { collectionId: { _eq: \"342f12106eab6d904c-YOUDLEBACKGROUND\" } }}
        {children: { collectionId: { _eq: \"342f12106eab6d904c-YOUDLECHEST\" } }}
        {children: { collectionId: { _eq: \"342f12106eab6d904c-YOUDLESKIN\" } }}
      ]
    },

    order_by: {
    	id: asc
  	}
  ) {
		id
    owner
  }

  og_youdles: nfts(
    	where: {
        collectionId: {_eq: \"36af143c6012f6266b-YOUDLE\"}
      }

  ) {
    id
    owner
    metadata_properties
  }

}",
};
