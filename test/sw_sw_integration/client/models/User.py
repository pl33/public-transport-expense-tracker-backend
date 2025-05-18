from typing import *

from pydantic import BaseModel, Field


class User(BaseModel):
    """
    None model

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    id: Optional[int] = Field(validation_alias="id", default=None)

    jwt_issuer: Optional[str] = Field(validation_alias="jwt_issuer", default=None)

    jwt_subject: Optional[str] = Field(validation_alias="jwt_subject", default=None)

    name: Optional[str] = Field(validation_alias="name", default=None)
